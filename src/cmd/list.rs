use std::collections::BTreeMap;

use anyhow::Result;

use crate::config::Config;
use crate::repo_source::{self, Repo};

// ANSI helpers
const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";
const BOLD: &str = "\x1b[1m";

const GROUP_COLORS: &[&str] = &[
    "\x1b[48;5;24m\x1b[97m",  // dark blue
    "\x1b[48;5;22m\x1b[97m",  // dark green
    "\x1b[48;5;94m\x1b[97m",  // dark orange
    "\x1b[48;5;30m\x1b[97m",  // teal
    "\x1b[48;5;238m\x1b[97m", // dark gray
];

pub fn run(raw: bool) -> Result<()> {
    let config = Config::load()?;
    let repos = repo_source::scan(&config.base_dirs())?;
    let has_multiple_bases = config.base_dirs().len() > 1;

    if raw {
        for repo in &repos {
            println!("{}", repo.path.display());
        }
        return Ok(());
    }

    // Pretty print: grouped by base/host
    let mut grouped: Vec<(String, Vec<&Repo>)> = Vec::new();
    let mut group_index: BTreeMap<String, usize> = BTreeMap::new();

    for repo in &repos {
        let key = group_key(repo, has_multiple_bases);
        if let Some(&idx) = group_index.get(&key) {
            grouped[idx].1.push(repo);
        } else {
            let idx = grouped.len();
            group_index.insert(key.clone(), idx);
            grouped.push((key, vec![repo]));
        }
    }

    for (i, (label, group_repos)) in grouped.iter().enumerate() {
        let color = GROUP_COLORS[i % GROUP_COLORS.len()];
        println!(
            "{} {} {} {}({}){}",
            color,
            label,
            RESET,
            DIM,
            group_repos.len(),
            RESET
        );
        for repo in group_repos {
            println!(
                "  {}{}/{}{}  {}{}{}",
                BOLD,
                repo.owner,
                repo.name,
                RESET,
                DIM,
                repo.git_url(),
                RESET
            );
        }
    }

    println!("\n{}Total: {} repositories{}", DIM, repos.len(), RESET);

    Ok(())
}

fn group_key(repo: &Repo, has_multiple_bases: bool) -> String {
    if has_multiple_bases {
        let base_name = repo
            .base
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        format!("{}/{}", base_name, repo.host)
    } else {
        repo.host.clone()
    }
}
