use std::collections::HashMap;

use anyhow::Result;
use dialoguer::Input;

use crate::config::{BaseDir, Config, HookEntry, config_exists, config_path};
use crate::repo_source;
use crate::task;

pub fn run() -> Result<()> {
    let is_new = !config_exists();

    let config = if is_new {
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
            tasks: HashMap::default(),
            hooks: vec![HookEntry {
                event: "post_add".to_string(),
                matcher: None,
                tasks: vec!["zoxide".to_string()],
                env: HashMap::default(),
            }],
        };
        config.save()?;
        eprintln!("✅ Config saved to {}", config_path().display());
        config
    } else {
        eprintln!("📁 Config already exists at {}", config_path().display());
        Config::load()?
    };

    task::install_builtins()?;

    // Show what was configured
    let repos = repo_source::scan(&config.base_dirs())?;
    eprintln!(
        "📊 {} base directories, {} repositories found",
        config.base_dirs().len(),
        repos.len()
    );
    if !config.hooks.is_empty() {
        eprintln!("🪝 {} hooks configured", config.hooks.len());
    }
    if !config.tasks.is_empty() {
        eprintln!("📜 {} tasks configured", config.tasks.len());
    }

    eprintln!();
    eprintln!("💡 Tip: add to ~/.zshrc (or ~/.bashrc):");
    eprintln!();
    eprintln!("   eval \"$(projj shell-setup zsh)\"");
    eprintln!();

    Ok(())
}
