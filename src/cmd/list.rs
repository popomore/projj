use anyhow::Result;

use crate::config::Config;
use crate::repo_source;

pub fn run() -> Result<()> {
    let config = Config::load()?;
    let repos = repo_source::scan(&config.base_dirs())?;

    for repo in &repos {
        println!("{}", repo.path.display());
    }

    Ok(())
}
