use anyhow::Result;

use crate::config::Config;
use crate::hook;
use crate::repo_source;

pub fn run(script: &str, all: bool, match_pattern: Option<&str>) -> Result<()> {
    let config = Config::load()?;
    let resolved_owned = config.resolve_script(script);
    let resolved = resolved_owned.as_str();

    if all {
        let repos = repo_source::scan(&config.base_dirs())?;

        // Filter by --match if provided
        let matcher = match match_pattern {
            Some(p) => Some(
                regex_lite::Regex::new(p)
                    .map_err(|e| anyhow::anyhow!("Invalid --match pattern '{}': {}", p, e))?,
            ),
            None => None,
        };
        let filtered: Vec<_> = repos
            .iter()
            .filter(|r| {
                matcher
                    .as_ref()
                    .map(|re| re.is_match(&r.display_key()))
                    .unwrap_or(true)
            })
            .collect();

        let total = filtered.len();
        for (i, repo) in filtered.iter().enumerate() {
            eprintln!("[{}/{}] {}", i + 1, total, repo.display_key());
            if let Err(e) = hook::run_command(resolved, &repo.path) {
                eprintln!("  Error: {}", e);
            }
        }
    } else {
        let cwd = std::env::current_dir()?;
        hook::run_command(resolved, &cwd)?;
    }

    Ok(())
}
