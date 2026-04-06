use anyhow::Result;

use super::search;
use crate::config::Config;
use crate::repo_source::{self, Repo};

pub fn run(keyword: Option<&str>) -> Result<()> {
    let config = Config::load()?;
    let repos = repo_source::scan(&config.base_dirs())?;
    let has_multiple_bases = config.base_dirs().len() > 1;

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
        search::print_repo_info(repo, has_multiple_bases);
        println!("{}", repo.path.display());
        return Ok(());
    }

    let display_items = search::build_indexed_items(&matched, has_multiple_bases);
    let selected = search::fzf_indexed(&display_items, keyword)?;
    match selected {
        Some(idx) => println!("{}", matched[idx].path.display()),
        None => std::process::exit(1),
    }

    Ok(())
}
