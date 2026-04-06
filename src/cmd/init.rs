use std::collections::HashMap;

use anyhow::Result;
use dialoguer::Input;

use crate::config::{BaseDir, Config, HookEntry, config_exists, config_path};
use crate::hook;
use crate::repo_source;

pub fn run() -> Result<()> {
    let config = if config_exists() {
        eprintln!("Config already exists at {}", config_path().display());
        eprintln!("Loading existing config...");
        Config::load()?
    } else {
        let home = dirs::home_dir().unwrap_or_default();
        let default_base = home.join("projj").to_string_lossy().to_string();

        let base: String = Input::new()
            .with_prompt("Set base directory")
            .default(default_base)
            .interact_text()?;

        let platform: String = Input::new()
            .with_prompt("Default platform")
            .default("github.com".to_string())
            .interact_text()?;

        let config = Config {
            base: BaseDir::Single(base),
            platform,
            scripts: HashMap::default(),
            hooks: vec![HookEntry {
                event: "post_add".to_string(),
                matcher: None,
                command: "zoxide".to_string(),
                env: HashMap::default(),
            }],
        };
        config.save()?;
        eprintln!("Config saved to {}", config_path().display());
        config
    };

    hook::install_builtin_hooks()?;

    let repos = repo_source::scan(&config.base_dirs())?;
    if !repos.is_empty() {
        eprintln!("Syncing {} repositories...", repos.len());
        for repo in &repos {
            let repo_key = repo.display_key();
            let _ = hook::run_hooks(&config, "post_add", &repo_key, &repo.path);
        }
        eprintln!("Done.");
    }

    eprintln!();
    eprintln!("Tip: add to ~/.zshrc (or ~/.bashrc):");
    eprintln!();
    eprintln!("  eval \"$(projj shell-setup zsh)\"");
    eprintln!();

    Ok(())
}
