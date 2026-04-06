use anyhow::Result;

use crate::color::{DIM, RESET, color};
use crate::config::Config;
use crate::repo_source;
use crate::task;

pub fn run(script: &str, all: bool, match_pattern: Option<&str>, args: &[String]) -> Result<()> {
    let config = Config::load()?;
    let resolved_owned = task::resolve(&config, script);
    let resolved = if args.is_empty() {
        resolved_owned
    } else {
        format!("{} {}", resolved_owned, args.join(" "))
    };

    if all {
        let repos = repo_source::scan(&config.base_dirs())?;

        // Filter by --match if provided
        let matcher = match match_pattern {
            Some(p) => Some(
                regex_lite::Regex::new(p)
                    .map_err(|e| anyhow::anyhow!("Invalid --match pattern '{p}': {e}"))?,
            ),
            None => None,
        };
        let filtered: Vec<_> = repos
            .iter()
            .filter(|r| {
                matcher
                    .as_ref()
                    .is_none_or(|re| re.is_match(&r.display_key()))
            })
            .collect();

        let total = filtered.len();
        for (i, repo) in filtered.iter().enumerate() {
            let d = color(DIM);
            let r = color(RESET);
            eprintln!("{d}[{}/{}]{r} 📦 {}", i + 1, total, repo.display_key());
            if let Err(e) = task::run(&resolved, &repo.path) {
                eprintln!("  ❌ {e}");
            }
        }
    } else {
        let cwd = std::env::current_dir()?;
        task::run(&resolved, &cwd)?;
    }

    Ok(())
}
