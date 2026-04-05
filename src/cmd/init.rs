use anyhow::Result;
use dialoguer::Input;

use crate::config::{BaseDir, Config, config_exists, config_path, hooks_dir};
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
            scripts: Default::default(),
            hooks: Default::default(),
        };
        config.save()?;
        eprintln!("Config saved to {}", config_path().display());
        config
    };

    // Install built-in hooks if hooks dir doesn't exist
    install_builtin_hooks()?;

    // Sync existing repos by running post_add hooks for each
    let repos = repo_source::scan(&config.base_dirs())?;
    if !repos.is_empty() {
        eprintln!("Syncing {} repositories...", repos.len());
        for repo in &repos {
            let repo_key = repo.display_key();
            let _ = hook::run_hooks(&config, "post_add", &repo_key, &repo.path);
        }
        eprintln!("Done.");
    }

    // Shell integration hint
    eprintln!();
    eprintln!("Tip: add to ~/.zshrc:");
    eprintln!();
    eprintln!("  eval \"$(projj completions zsh)\"");
    eprintln!();
    eprintln!("  p() {{");
    eprintln!("    local dir");
    eprintln!("    dir=$(projj find \"$@\")");
    eprintln!("    [ -n \"$dir\" ] && cd \"$dir\"");
    eprintln!("  }}");
    eprintln!();

    Ok(())
}

/// Install built-in hook scripts to ~/.projj/hooks/
fn install_builtin_hooks() -> Result<()> {
    let dir = hooks_dir();
    std::fs::create_dir_all(&dir)?;

    let builtins: &[(&str, &[u8])] = &[
        ("zoxide", include_bytes!("../../hooks/zoxide")),
        (
            "git_config_user",
            include_bytes!("../../hooks/git_config_user"),
        ),
    ];

    for (name, content) in builtins {
        let path = dir.join(name);
        if !path.exists() {
            std::fs::write(&path, content)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))?;
            }
            eprintln!("Installed hook: {}", path.display());
        }
    }

    Ok(())
}
