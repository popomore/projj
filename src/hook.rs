use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Result, bail};

use crate::config::{Config, HookEntry};

/// Repo context passed to hooks via stdin JSON and env vars.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RepoContext {
    pub event: String,
    pub repo: RepoInfo,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RepoInfo {
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

    let context = RepoContext {
        event: event.to_string(),
        repo: RepoInfo {
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
    if let Ok(ctx) = serde_json::from_str::<RepoContext>(stdin_json) {
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
