use anyhow::Result;

use crate::config::Config;
use crate::git;
use crate::hook;

pub fn run(repo_arg: &str) -> Result<()> {
    let config = Config::load()?;
    let info = git::parse_repo(repo_arg, &config.platform)?;

    let base = config.choose_base()?;
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
