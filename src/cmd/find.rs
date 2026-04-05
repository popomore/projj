use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::Result;

use crate::config::Config;
use crate::integration;
use crate::repo_source::{self, Repo};

pub fn run(keyword: Option<&str>) -> Result<()> {
    let config = Config::load()?;
    let repos = repo_source::scan(&config.base_dirs())?;
    let base_dirs = config.base_dirs();
    let has_multiple_bases = base_dirs.len() > 1;

    let matched: Vec<Repo> = if let Some(kw) = keyword {
        let m = repo_source::find(&repos, kw);
        if m.is_empty() {
            eprintln!("No repository found matching: {kw}");
            std::process::exit(1);
        }
        m
    } else {
        repos
    };

    if matched.is_empty() {
        eprintln!("No repositories found");
        std::process::exit(1);
    }

    if matched.len() == 1 {
        let repo = &matched[0];
        print_repo_info(repo, has_multiple_bases);
        println!("{}", repo.path.display());
        return Ok(());
    }

    let (display_items, path_map) = build_display_items(&matched, has_multiple_bases);

    let selected = integration::select_one(&display_items, "Select repository", keyword)?;
    match selected {
        Some(display) => {
            if let Some(path) = path_map.get(&display) {
                println!("{}", path.display());
            } else {
                // Selected a header line, not a repo
                eprintln!("No repository selected");
                std::process::exit(1);
            }
        }
        None => std::process::exit(1),
    }

    Ok(())
}

fn print_repo_info(repo: &Repo, has_multiple_bases: bool) {
    let group_label = if has_multiple_bases {
        let base_name = repo
            .base
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        format!("{}/{}", base_name, repo.host)
    } else {
        repo.host.clone()
    };
    let color = BASE_COLORS[0];
    eprintln!("{color} {group_label} {RESET}");
    eprintln!(
        "    {}  {}{}{}",
        repo.short_key(),
        DIM,
        repo.git_url(),
        RESET
    );
}

// ANSI helpers
const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";

// Background colors for base groups
const BASE_COLORS: &[&str] = &[
    "\x1b[48;5;24m\x1b[97m",  // dark blue
    "\x1b[48;5;22m\x1b[97m",  // dark green
    "\x1b[48;5;94m\x1b[97m",  // dark orange
    "\x1b[48;5;30m\x1b[97m",  // teal
    "\x1b[48;5;238m\x1b[97m", // dark gray
];

/// Build display items with group tag on every line:
///
/// ```text
///  github.com  popomore/projj    git@github.com:popomore/projj.git
/// ```
///
/// Group tag (colored) + owner/repo + git URL (dimmed), all on one line.
/// This ensures group context is preserved when fzf filters results.
fn build_display_items(
    repos: &[Repo],
    has_multiple_bases: bool,
) -> (Vec<String>, BTreeMap<String, PathBuf>) {
    let mut path_map = BTreeMap::new();

    // Assign color per group
    let mut group_colors: BTreeMap<String, &str> = BTreeMap::new();
    let mut color_idx = 0;

    for repo in repos {
        let group_key = group_key_for(repo, has_multiple_bases);
        if let std::collections::btree_map::Entry::Vacant(e) = group_colors.entry(group_key) {
            e.insert(BASE_COLORS[color_idx % BASE_COLORS.len()]);
            color_idx += 1;
        }
    }

    let mut items = Vec::new();

    for repo in repos {
        let group_key = group_key_for(repo, has_multiple_bases);
        let color = group_colors[&group_key];
        let display = format!(
            "{} {} {} {}  {}{}{}",
            color,
            group_key,
            RESET,
            repo.short_key(),
            DIM,
            repo.git_url(),
            RESET,
        );
        path_map.insert(display.clone(), repo.path.clone());
        items.push(display);
    }

    (items, path_map)
}

fn group_key_for(repo: &Repo, has_multiple_bases: bool) -> String {
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
