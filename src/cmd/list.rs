use anyhow::Result;

use crate::config::Config;
use crate::repo_source;
use crate::select;

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

    select::print_grouped_list(&repos, has_multiple_bases);

    Ok(())
}
