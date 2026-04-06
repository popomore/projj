use std::path::PathBuf;

use anyhow::Result;

use crate::config::Config;
use crate::git;
use crate::hook;
use crate::select;

/// Choose a base directory. Returns directly if only one,
/// otherwise prompts user to select via fzf/dialoguer.
fn choose_base(config: &Config) -> Result<PathBuf> {
    let dirs = config.base_dirs();
    if dirs.len() == 1 {
        return Ok(dirs[0].clone());
    }
    let choices: Vec<String> = dirs
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    let selected = select::select_one(&choices, "Choose base directory", None)?;
    match selected {
        Some(s) => Ok(PathBuf::from(s)),
        None => anyhow::bail!("No base directory selected"),
    }
}

pub fn run(repo_arg: &str) -> Result<()> {
    let config = Config::load()?;
    let info = git::parse_repo(repo_arg, &config.platform)?;

    let base = choose_base(&config)?;
    let target = base.join(info.rel_path());
    let repo_key = info.rel_path();

    if target.exists() {
        eprintln!("{} already exists", target.display());
        hook::run_hooks(&config, "post_add", &repo_key, &target)?;
        println!("{}", target.display());
        return Ok(());
    }

    let is_local = repo_arg.starts_with('.') || repo_arg.starts_with('/');

    hook::run_hooks(&config, "pre_add", &repo_key, &target)?;

    if is_local {
        git::move_local(repo_arg, &target)?;
    } else {
        git::clone_remote(&info.clone_url, &target)?;
    }

    hook::run_hooks(&config, "post_add", &repo_key, &target)?;

    eprintln!("Added {}", info.rel_path());
    println!("{}", target.display());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::BaseDir;
    use std::collections::HashMap;

    #[test]
    fn test_choose_base_single() {
        let config = Config {
            base: BaseDir::Single("/tmp/projj".to_string()),
            platform: "github.com".to_string(),
            scripts: HashMap::new(),
            hooks: vec![],
        };
        let base = choose_base(&config).unwrap();
        assert_eq!(base, PathBuf::from("/tmp/projj"));
    }
}
