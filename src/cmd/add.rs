use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Result, bail};

use super::search;
use crate::config::Config;
use crate::git;
use crate::hook;

pub fn run(repo_arg: &str) -> Result<()> {
    let config = Config::load()?;
    let info = git::parse_repo(repo_arg, &config.platform)?;

    let base_dirs = config.base_dirs();
    let base = choose_base(&base_dirs)?;
    let target = base.join(info.rel_path());
    let repo_key = info.rel_path();

    if target.exists() {
        eprintln!("{} already exists", target.display());
        // Still run post_add hooks (e.g. zoxide registration)
        hook::run_hooks(&config, "post_add", &repo_key, &target)?;
        println!("{}", target.display());
        return Ok(());
    }

    let is_local = repo_arg.starts_with('.') || repo_arg.starts_with('/');

    // pre_add hooks
    hook::run_hooks(&config, "pre_add", &repo_key, &target)?;

    if is_local {
        move_local(repo_arg, &target)?;
    } else {
        clone_remote(&info.clone_url, &target)?;
    }

    // post_add hooks (zoxide, git_config_user, etc.)
    hook::run_hooks(&config, "post_add", &repo_key, &target)?;

    eprintln!("Added {}", info.rel_path());
    println!("{}", target.display());

    Ok(())
}

fn choose_base(base_dirs: &[PathBuf]) -> Result<PathBuf> {
    if base_dirs.len() == 1 {
        return Ok(base_dirs[0].clone());
    }

    let choices: Vec<String> = base_dirs
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    let selected = search::select_one(&choices, "Choose base directory", None)?;
    match selected {
        Some(s) => Ok(PathBuf::from(s)),
        None => bail!("No base directory selected"),
    }
}

fn clone_remote(url: &str, target: &Path) -> Result<()> {
    eprintln!("Cloning {} into {}", url, target.display());
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let status = Command::new("git")
        .args(["clone", "--progress", url, &target.to_string_lossy()])
        .stdin(std::process::Stdio::null())
        .status()?;
    if !status.success() {
        bail!("git clone failed");
    }
    Ok(())
}

fn move_local(src: &str, target: &Path) -> Result<()> {
    let src = Path::new(src).canonicalize()?;
    eprintln!("Moving {} to {}", src.display(), target.display());
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::rename(&src, target)?;
    Ok(())
}
