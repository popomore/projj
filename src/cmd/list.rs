use std::collections::BTreeMap;

use anyhow::Result;

use super::search::{self, BOLD, DIM, GROUP_COLORS, RESET};
use crate::config::Config;
use crate::repo_source::{self, Repo};

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
        let key = search::group_key_for(repo, has_multiple_bases);
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

    println!("\n{DIM}Total: {} repositories{RESET}", repos.len());

    Ok(())
}
