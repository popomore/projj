use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Result, bail};

use crate::config::{Config, HookEntry};

/// Repo context passed to hooks via stdin JSON and env vars.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct HookContext {
    pub event: String,
    pub repo: HookRepoInfo,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct HookRepoInfo {
    pub path: String,
    pub host: String,
    pub owner: String,
    pub name: String,
    pub git_url: String,
}

/// Run all hooks matching an event for a given repo.
pub fn run_hooks(config: &Config, event: &str, repo_key: &str, cwd: &Path) -> Result<()> {
    let matching: Vec<&HookEntry> = config
        .hooks
        .iter()
        .filter(|h| h.event == event)
        .filter(|h| matches_repo(h.matcher.as_deref(), repo_key))
        .collect();

    if matching.is_empty() {
        return Ok(());
    }

    // Build repo context from repo_key (host/owner/name)
    let parts: Vec<&str> = repo_key.splitn(3, '/').collect();
    let (host, owner, name) = if parts.len() == 3 {
        (parts[0], parts[1], parts[2])
    } else {
        ("", "", repo_key)
    };

    let context = HookContext {
        event: event.to_string(),
        repo: HookRepoInfo {
            path: cwd.to_string_lossy().to_string(),
            host: host.to_string(),
            owner: owner.to_string(),
            name: name.to_string(),
            git_url: if host.is_empty() {
                String::new()
            } else {
                format!("git@{host}:{owner}/{name}.git")
            },
        },
    };

    let stdin_json = serde_json::to_string(&context)?;

    for hook in matching {
        let script = config.resolve_script(&hook.command);
        eprintln!("  hook [{}] {}", event, &script);
        run_script(&script, &hook.env, &stdin_json, cwd)?;
    }

    Ok(())
}

/// Build a shell command appropriate for the current OS.
fn shell_command(script: &str) -> Command {
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

/// Run a shell script with env vars and stdin JSON.
fn run_script(
    script: &str,
    extra_env: &std::collections::HashMap<String, String>,
    stdin_json: &str,
    cwd: &Path,
) -> Result<()> {
    let mut cmd = shell_command(script);
    cmd.stdin(Stdio::piped());

    if cwd.exists() {
        cmd.current_dir(cwd);
    }

    // Set env vars from hook entry
    for (k, v) in extra_env {
        cmd.env(k, v);
    }

    // Parse stdin JSON to also set PROJJ_* env vars
    if let Ok(ctx) = serde_json::from_str::<HookContext>(stdin_json) {
        cmd.env("PROJJ_EVENT", &ctx.event);
        cmd.env("PROJJ_REPO_PATH", &ctx.repo.path);
        cmd.env("PROJJ_REPO_HOST", &ctx.repo.host);
        cmd.env("PROJJ_REPO_OWNER", &ctx.repo.owner);
        cmd.env("PROJJ_REPO_NAME", &ctx.repo.name);
        cmd.env("PROJJ_REPO_URL", &ctx.repo.git_url);
    }

    let mut child = cmd.spawn()?;

    // Write JSON to stdin
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(stdin_json.as_bytes());
    }

    let status = child.wait()?;
    if !status.success() {
        bail!(
            "Hook failed with exit code: {}",
            status.code().unwrap_or(-1)
        );
    }
    Ok(())
}

/// Check if `repo_key` matches a matcher regex. None/empty = match all.
fn matches_repo(matcher: Option<&str>, repo_key: &str) -> bool {
    match matcher {
        None | Some("" | "*") => true,
        Some(pattern) => regex_lite::Regex::new(pattern)
            .map(|re| re.is_match(repo_key))
            .unwrap_or(false),
    }
}

/// Run a raw command (for `projj run`), no hook context.
pub fn run_command(script: &str, cwd: &Path) -> Result<()> {
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

/// Install built-in hook scripts to `~/.projj/hooks/`.
/// Only writes files that don't already exist.
pub fn install_builtin_hooks() -> Result<()> {
    let dir = crate::config::hooks_dir();
    std::fs::create_dir_all(&dir)?;

    let builtins: &[(&str, &[u8])] = &[
        ("zoxide", include_bytes!("../hooks/zoxide")),
        (
            "git_config_user",
            include_bytes!("../hooks/git_config_user"),
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::config::{Config, HookEntry};

    #[test]
    fn test_matches_repo_none() {
        assert!(matches_repo(None, "github.com/popomore/projj"));
    }

    #[test]
    fn test_matches_repo_empty() {
        assert!(matches_repo(Some(""), "github.com/popomore/projj"));
    }

    #[test]
    fn test_matches_repo_wildcard() {
        assert!(matches_repo(Some("*"), "github.com/popomore/projj"));
    }

    #[test]
    fn test_matches_repo_exact_host() {
        assert!(matches_repo(
            Some("github\\.com"),
            "github.com/popomore/projj"
        ));
        assert!(!matches_repo(
            Some("gitlab\\.com"),
            "github.com/popomore/projj"
        ));
    }

    #[test]
    fn test_matches_repo_owner_pattern() {
        assert!(matches_repo(
            Some("github\\.com/SeeleAI"),
            "github.com/SeeleAI/agent"
        ));
        assert!(!matches_repo(
            Some("github\\.com/SeeleAI"),
            "github.com/popomore/projj"
        ));
    }

    #[test]
    fn test_matches_repo_multi_pattern() {
        let pattern = "gitlab\\.alibaba|code\\.alipay";
        assert!(matches_repo(
            Some(pattern),
            "gitlab.alibaba-inc.com/team/repo"
        ));
        assert!(matches_repo(Some(pattern), "code.alipay.com/team/repo"));
        assert!(!matches_repo(Some(pattern), "github.com/team/repo"));
    }

    #[test]
    fn test_matches_repo_invalid_regex() {
        assert!(!matches_repo(Some("[invalid"), "github.com/popomore/projj"));
    }

    #[test]
    fn test_run_command_success() {
        let dir = tempfile::tempdir().unwrap();
        let result = run_command("true", dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_command_failure() {
        let dir = tempfile::tempdir().unwrap();
        let result = run_command("false", dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_run_command_nonexistent_cwd() {
        // cwd doesn't exist, command should still run (in current dir)
        let result = run_command("true", Path::new("/nonexistent"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_hooks_no_matching() {
        let config = Config {
            base: crate::config::BaseDir::Single("/tmp".to_string()),
            platform: "github.com".to_string(),
            scripts: HashMap::new(),
            hooks: vec![HookEntry {
                event: "post_add".to_string(),
                matcher: Some("gitlab\\.com".to_string()),
                command: "echo hi".to_string(),
                env: HashMap::new(),
            }],
        };
        let dir = tempfile::tempdir().unwrap();
        // github.com repo won't match gitlab\.com matcher
        let result = run_hooks(&config, "post_add", "github.com/popomore/projj", dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_hooks_no_hooks_for_event() {
        let config = Config {
            base: crate::config::BaseDir::Single("/tmp".to_string()),
            platform: "github.com".to_string(),
            scripts: HashMap::new(),
            hooks: vec![],
        };
        let dir = tempfile::tempdir().unwrap();
        let result = run_hooks(&config, "post_add", "github.com/popomore/projj", dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_hooks_matching() {
        let dir = tempfile::tempdir().unwrap();
        let marker = dir.path().join("hook_ran");
        let config = Config {
            base: crate::config::BaseDir::Single("/tmp".to_string()),
            platform: "github.com".to_string(),
            scripts: HashMap::new(),
            hooks: vec![HookEntry {
                event: "post_add".to_string(),
                matcher: Some("github\\.com".to_string()),
                command: format!("touch {}", marker.display()),
                env: HashMap::new(),
            }],
        };
        let result = run_hooks(&config, "post_add", "github.com/popomore/projj", dir.path());
        assert!(result.is_ok());
        assert!(marker.exists());
    }

    #[cfg(unix)]
    #[test]
    fn test_run_hooks_with_env() {
        let dir = tempfile::tempdir().unwrap();
        let outfile = dir.path().join("env_out");
        let mut env = HashMap::new();
        env.insert("MY_VAR".to_string(), "hello".to_string());
        let config = Config {
            base: crate::config::BaseDir::Single("/tmp".to_string()),
            platform: "github.com".to_string(),
            scripts: HashMap::new(),
            hooks: vec![HookEntry {
                event: "post_add".to_string(),
                matcher: None,
                command: format!("echo $MY_VAR > {}", outfile.display()),
                env,
            }],
        };
        let result = run_hooks(&config, "post_add", "github.com/popomore/projj", dir.path());
        assert!(result.is_ok());
        let content = std::fs::read_to_string(&outfile).unwrap();
        assert_eq!(content.trim(), "hello");
    }

    #[cfg(windows)]
    #[test]
    fn test_run_hooks_with_env_windows() {
        let dir = tempfile::tempdir().unwrap();
        let outfile = dir.path().join("env_out.txt");
        let mut env = HashMap::new();
        env.insert("MY_VAR".to_string(), "hello".to_string());
        let config = Config {
            base: crate::config::BaseDir::Single("C:\\tmp".to_string()),
            platform: "github.com".to_string(),
            scripts: HashMap::new(),
            hooks: vec![HookEntry {
                event: "post_add".to_string(),
                matcher: None,
                command: format!("echo %MY_VAR% > {}", outfile.display()),
                env,
            }],
        };
        let result = run_hooks(&config, "post_add", "github.com/popomore/projj", dir.path());
        assert!(result.is_ok());
        let content = std::fs::read_to_string(&outfile).unwrap();
        assert!(content.trim().contains("hello"));
    }

    #[cfg(unix)]
    #[test]
    fn test_run_hooks_receives_projj_env_vars() {
        let dir = tempfile::tempdir().unwrap();
        let outfile = dir.path().join("env_out");
        let config = Config {
            base: crate::config::BaseDir::Single("/tmp".to_string()),
            platform: "github.com".to_string(),
            scripts: HashMap::new(),
            hooks: vec![HookEntry {
                event: "post_add".to_string(),
                matcher: None,
                command: format!("echo $PROJJ_REPO_HOST > {}", outfile.display()),
                env: HashMap::new(),
            }],
        };
        let result = run_hooks(&config, "post_add", "github.com/popomore/projj", dir.path());
        assert!(result.is_ok());
        let content = std::fs::read_to_string(&outfile).unwrap();
        assert_eq!(content.trim(), "github.com");
    }

    #[cfg(windows)]
    #[test]
    fn test_run_hooks_receives_projj_env_vars_windows() {
        let dir = tempfile::tempdir().unwrap();
        let outfile = dir.path().join("env_out.txt");
        let config = Config {
            base: crate::config::BaseDir::Single("C:\\tmp".to_string()),
            platform: "github.com".to_string(),
            scripts: HashMap::new(),
            hooks: vec![HookEntry {
                event: "post_add".to_string(),
                matcher: None,
                command: format!("echo %PROJJ_REPO_HOST% > {}", outfile.display()),
                env: HashMap::new(),
            }],
        };
        let result = run_hooks(&config, "post_add", "github.com/popomore/projj", dir.path());
        assert!(result.is_ok());
        let content = std::fs::read_to_string(&outfile).unwrap();
        assert!(content.trim().contains("github.com"));
    }

    #[test]
    fn test_run_hooks_with_script_resolve() {
        let dir = tempfile::tempdir().unwrap();
        let marker = dir.path().join("script_ran");
        let mut scripts = HashMap::new();
        scripts.insert(
            "myscript".to_string(),
            format!("touch {}", marker.display()),
        );
        let config = Config {
            base: crate::config::BaseDir::Single("/tmp".to_string()),
            platform: "github.com".to_string(),
            scripts,
            hooks: vec![HookEntry {
                event: "post_add".to_string(),
                matcher: None,
                command: "myscript".to_string(),
                env: HashMap::new(),
            }],
        };
        let result = run_hooks(&config, "post_add", "github.com/popomore/projj", dir.path());
        assert!(result.is_ok());
        assert!(marker.exists());
    }

    #[test]
    fn test_run_hooks_simple_repo_key() {
        // repo_key without host/owner/name format
        let dir = tempfile::tempdir().unwrap();
        let config = Config {
            base: crate::config::BaseDir::Single("/tmp".to_string()),
            platform: "github.com".to_string(),
            scripts: HashMap::new(),
            hooks: vec![HookEntry {
                event: "post_add".to_string(),
                matcher: None,
                command: "true".to_string(),
                env: HashMap::new(),
            }],
        };
        let result = run_hooks(&config, "post_add", "simple-key", dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_hooks_hook_failure() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config {
            base: crate::config::BaseDir::Single("/tmp".to_string()),
            platform: "github.com".to_string(),
            scripts: HashMap::new(),
            hooks: vec![HookEntry {
                event: "post_add".to_string(),
                matcher: None,
                command: "false".to_string(),
                env: HashMap::new(),
            }],
        };
        let result = run_hooks(&config, "post_add", "github.com/popomore/projj", dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_shell_command() {
        let cmd = shell_command("echo hi");
        let program = cmd.get_program().to_string_lossy().to_string();
        if cfg!(windows) {
            assert_eq!(program, "cmd");
        } else {
            assert_eq!(program, "sh");
        }
    }
}
