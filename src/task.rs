use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Result, bail};

use crate::config::Config;

/// Directory for task scripts: `~/.projj/tasks/`
pub fn tasks_dir() -> PathBuf {
    crate::config::config_dir().join("tasks")
}

/// Resolve a task name with three-level lookup:
/// 1. `[tasks]` table in config
/// 2. Executable file in `~/.projj/tasks/`
/// 3. Raw command as-is
pub fn resolve(config: &Config, name: &str) -> String {
    // 1. Tasks table
    if let Some(task) = config.tasks.get(name) {
        return task.clone();
    }

    // 2. ~/.projj/tasks/ directory (only for simple names without spaces/slashes)
    if !name.contains(' ') && !name.contains('/') {
        let task_path = tasks_dir().join(name);
        if task_path.exists() {
            return task_path.to_string_lossy().to_string();
        }
    }

    // 3. Raw command
    name.to_string()
}

/// Build a shell command appropriate for the current OS.
pub fn shell_command(script: &str) -> Command {
    if cfg!(windows) {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", script]);
        cmd
    } else {
        let mut cmd = Command::new("sh");
        cmd.args(["-c", script]);
        cmd
    }
}

/// Run a command in a directory.
pub fn run(script: &str, cwd: &Path) -> Result<()> {
    let mut cmd = shell_command(script);

    if cwd.exists() {
        cmd.current_dir(cwd);
    }

    let status = cmd.status()?;
    if !status.success() {
        bail!(
            "Command failed with exit code: {}",
            status.code().unwrap_or(-1)
        );
    }
    Ok(())
}

/// Include a task file. Name is derived from the file name.
macro_rules! builtin {
    ($file:literal) => {
        ($file, include_bytes!(concat!("../tasks/", $file)))
    };
}

/// Registry of built-in tasks. Add new entries here.
///
/// `repo-status` is retained as a deprecation shim that execs `status`; it
/// will be removed in a future release.
const BUILTINS: &[(&str, &[u8])] = &[
    builtin!("clean"),
    builtin!("git-config-user"),
    builtin!("repo-status"),
    builtin!("status"),
    builtin!("zoxide"),
];

/// Install built-in task scripts to `~/.projj/tasks/`.
/// Only writes files that don't already exist.
pub fn install_builtins() -> Result<()> {
    let dir = tasks_dir();
    std::fs::create_dir_all(&dir)?;

    for (name, content) in BUILTINS {
        let path = dir.join(name);
        if !path.exists() {
            std::fs::write(&path, content)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))?;
            }
            eprintln!("  📜 Installed task: {name}");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use crate::config::BaseDir;

    fn make_config() -> Config {
        Config {
            base: BaseDir::Single("/tmp".to_string()),
            platform: "github.com".to_string(),
            tasks: HashMap::new(),
            hooks: vec![],
        }
    }

    #[test]
    fn test_resolve_from_table() {
        let mut config = make_config();
        config
            .tasks
            .insert("clean".to_string(), "rm -rf node_modules".to_string());
        assert_eq!(resolve(&config, "clean"), "rm -rf node_modules");
    }

    #[test]
    fn test_resolve_raw_command() {
        let config = make_config();
        assert_eq!(resolve(&config, "git status"), "git status");
    }

    #[test]
    fn test_resolve_from_tasks_dir() {
        let config = make_config();
        // tasks_dir doesn't point to our temp dir, falls through to raw
        assert_eq!(resolve(&config, "mytask"), "mytask");
    }

    #[test]
    fn test_tasks_dir() {
        assert!(tasks_dir().ends_with("tasks"));
    }

    #[test]
    fn test_run_success() {
        let dir = tempfile::tempdir().unwrap();
        let result = run("true", dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_failure() {
        let dir = tempfile::tempdir().unwrap();
        let result = run("false", dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_run_nonexistent_cwd() {
        let result = run("true", Path::new("/nonexistent"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_shell_command_platform() {
        let cmd = shell_command("echo hi");
        let program = cmd.get_program().to_string_lossy().to_string();
        if cfg!(windows) {
            assert_eq!(program, "cmd");
        } else {
            assert_eq!(program, "sh");
        }
    }
}
