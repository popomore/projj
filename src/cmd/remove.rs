use std::fs;
use std::path::Path;

use anyhow::{Result, bail};
use dialoguer::Input;

use crate::config::Config;
use crate::hook;
use crate::repo_source;
use crate::select;

pub fn run(keyword: &str) -> Result<()> {
    let config = Config::load()?;
    let repos = repo_source::scan(&config.base_dirs())?;
    let matched = repo_source::find(&repos, keyword);

    if matched.is_empty() {
        bail!("No repository found matching: {keyword}");
    }

    let paths: Vec<String> = matched
        .iter()
        .map(|r| r.path.to_string_lossy().to_string())
        .collect();
    let selected = select::select_one(&paths, "Select repository to remove", Some(keyword))?;

    let Some(target) = selected else {
        bail!("No repository selected")
    };

    let repo = matched
        .iter()
        .find(|r| r.path.to_string_lossy() == target)
        .unwrap();

    let confirm_name = format!("{}/{}", repo.owner, repo.name);
    let repo_key = repo.display_key();

    eprintln!("🗑️  Will remove: {target}");
    eprintln!("⚠️  This cannot be undone!");

    let input: String = Input::new()
        .with_prompt(format!("Type '{confirm_name}' to confirm"))
        .interact_text()?;

    if input != confirm_name {
        eprintln!("❌ Cancelled.");
        return Ok(());
    }

    // pre_remove hooks
    hook::run_hooks(&config, "pre_remove", &repo_key, &repo.path)?;

    fs::remove_dir_all(&target)?;

    // Clean up empty parent directories
    let parent = Path::new(&target).parent();
    if let Some(p) = parent
        && p.read_dir().is_ok_and(|mut d| d.next().is_none())
    {
        let _ = fs::remove_dir(p);
    }

    // post_remove hooks (cwd is parent since repo dir is gone)
    let hook_cwd = Path::new(&target).parent().unwrap_or(Path::new("/"));
    hook::run_hooks(&config, "post_remove", &repo_key, hook_cwd)?;

    eprintln!("✅ Removed {repo_key}");

    Ok(())
}
