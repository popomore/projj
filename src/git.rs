use std::path::Path;
use std::process::Command;

use anyhow::{Result, bail};

/// Parsed git repository info.
#[derive(Debug, Clone)]
pub struct RepoInfo {
    pub host: String,
    pub owner: String,
    pub repo: String,
    pub clone_url: String,
}

impl RepoInfo {
    /// Relative path: host/owner/repo
    pub fn rel_path(&self) -> String {
        format!("{}/{}/{}", self.host, self.owner, self.repo)
    }
}

/// Clone a remote git repository with progress output (indented).
pub fn clone_remote(url: &str, target: &Path) -> Result<()> {
    eprintln!("📥 Cloning {url}");
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut child = Command::new("git")
        .args(["clone", "--progress", url, &target.to_string_lossy()])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    // Stream git's stderr with a 3-space indent per line.
    // Only \n starts a new indented line — \r is passed through unchanged so
    // that git's in-place progress overwrites (e.g. "Receiving objects: 45%\r")
    // render correctly in the terminal for both SSH and HTTPS remotes.
    if let Some(stderr) = child.stderr.take() {
        use std::io::{BufReader, Read};
        let mut reader = BufReader::new(stderr);
        let mut buf = [0u8; 256];
        let mut at_line_start = true;
        loop {
            let n = reader.read(&mut buf)?;
            if n == 0 {
                break;
            }
            for &b in &buf[..n] {
                if at_line_start {
                    eprint!("   ");
                    at_line_start = false;
                }
                eprint!("{}", b as char);
                if b == b'\n' {
                    at_line_start = true;
                }
            }
        }
    }

    let status = child.wait()?;
    if !status.success() {
        bail!("git clone failed");
    }
    Ok(())
}

/// Move a local git repository to the target path.
pub fn move_local(src: &str, target: &Path) -> Result<()> {
    let src = Path::new(src).canonicalize()?;
    eprintln!("📦 Moving {} to {}", src.display(), target.display());
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::rename(&src, target)?;
    Ok(())
}

/// Parse a repo argument into `RepoInfo`.
///
/// Supported formats:
/// - `git@github.com:owner/repo.git` (SSH)
/// - `ssh://git@host:port/owner/repo.git` (SSH with port)
/// - `https://github.com/owner/repo.git` (HTTPS)
/// - `https://192.168.1.1:8080/owner/repo.git` (HTTPS with IP and port)
/// - `owner/repo` (short form)
/// - `./path/to/local/repo` (local path)
pub fn parse_repo(input: &str, default_platform: &str) -> Result<RepoInfo> {
    // Local path
    if input.starts_with('.') || input.starts_with('/') || std::path::Path::new(input).is_absolute()
    {
        return parse_local_path(input);
    }

    // SSH with scheme: ssh://git@host:port/owner/repo.git
    if input.starts_with("ssh://") {
        return parse_ssh_scheme(input);
    }

    // SSH: git@host:owner/repo.git
    if input.starts_with("git@") {
        return parse_ssh(input);
    }

    // HTTPS: https://host[:port]/owner/repo[.git]
    if input.starts_with("https://") || input.starts_with("http://") {
        return parse_https(input);
    }

    // Short form: owner/repo
    if let Some((owner, repo)) = input.split_once('/')
        && !owner.is_empty()
        && !repo.is_empty()
        && !owner.contains('.')
    {
        let host = default_platform.to_string();
        let repo = repo.to_string();
        let owner = owner.to_string();
        let clone_url = format!("git@{host}:{owner}/{repo}.git");
        return Ok(RepoInfo {
            host,
            owner,
            repo,
            clone_url,
        });
    }

    bail!("Cannot parse repository: {input}")
}

/// Strip port from host string: "git.gitlab.cn:2224" → "git.gitlab.cn"
fn strip_port(host: &str) -> String {
    // If it looks like host:port (port is all digits), strip the port
    if let Some((h, port)) = host.rsplit_once(':')
        && port.chars().all(|c| c.is_ascii_digit())
    {
        return h.to_string();
    }
    host.to_string()
}

fn parse_ssh(input: &str) -> Result<RepoInfo> {
    // git@github.com:owner/repo.git
    // git@git.gitlab.cn:owner/repo.git
    let rest = input.strip_prefix("git@").unwrap();
    let (host_part, path) = rest
        .split_once(':')
        .ok_or_else(|| anyhow::anyhow!("Invalid SSH URL: {input}"))?;
    let path = path.strip_suffix(".git").unwrap_or(path);
    let (owner, repo) = path
        .split_once('/')
        .ok_or_else(|| anyhow::anyhow!("Invalid SSH URL: {input}"))?;
    Ok(RepoInfo {
        host: strip_port(host_part),
        owner: owner.to_string(),
        repo: repo.to_string(),
        clone_url: input.to_string(),
    })
}

fn parse_ssh_scheme(input: &str) -> Result<RepoInfo> {
    // ssh://git@git.gitlab.cn:2224/web/cms.git
    let without_scheme = input
        .strip_prefix("ssh://git@")
        .or_else(|| input.strip_prefix("ssh://"))
        .ok_or_else(|| anyhow::anyhow!("Invalid SSH URL: {input}"))?;

    // Split host:port from path
    // Could be: git.gitlab.cn:2224/web/cms.git
    // or:       git.gitlab.cn/web/cms.git
    let (host_part, path) = split_host_path(without_scheme)
        .ok_or_else(|| anyhow::anyhow!("Invalid SSH URL: {input}"))?;

    let path = path.strip_suffix(".git").unwrap_or(path);
    let (owner, repo) = path
        .split_once('/')
        .ok_or_else(|| anyhow::anyhow!("Invalid SSH URL: {input}"))?;

    Ok(RepoInfo {
        host: strip_port(host_part),
        owner: owner.to_string(),
        repo: repo.to_string(),
        clone_url: input.to_string(),
    })
}

fn parse_https(input: &str) -> Result<RepoInfo> {
    // https://github.com/owner/repo[.git]
    // https://192.168.1.1:8080/owner/repo.git
    let without_scheme = input
        .strip_prefix("https://")
        .or_else(|| input.strip_prefix("http://"))
        .unwrap();

    let (host_part, path) = split_host_path(without_scheme)
        .ok_or_else(|| anyhow::anyhow!("Invalid HTTPS URL: {input}"))?;

    let parts: Vec<&str> = path.splitn(2, '/').collect();
    if parts.len() < 2 {
        bail!("Invalid HTTPS URL: {input}");
    }
    let owner = parts[0].to_string();
    let repo = parts[1]
        .strip_suffix(".git")
        .unwrap_or(parts[1])
        .to_string();

    Ok(RepoInfo {
        host: strip_port(host_part),
        owner,
        repo,
        clone_url: input.to_string(),
    })
}

/// Split `host\[:port\]/rest/of/path` into `(host\[:port\], rest/of/path)`
fn split_host_path(s: &str) -> Option<(&str, &str)> {
    // Find the first '/' that comes after the host[:port]
    let slash_pos = s.find('/')?;
    let host_part = &s[..slash_pos];
    let path = &s[slash_pos + 1..];
    if host_part.is_empty() || path.is_empty() {
        return None;
    }
    Some((host_part, path))
}

fn parse_local_path(input: &str) -> Result<RepoInfo> {
    let path = Path::new(input)
        .canonicalize()
        .map_err(|_| anyhow::anyhow!("Local path does not exist: {input}"))?;

    let git_dir = path.join(".git");
    if !git_dir.exists() {
        bail!("Not a git repository: {input}");
    }

    let output = std::process::Command::new("git")
        .args(["config", "--get", "remote.origin.url"])
        .current_dir(&path)
        .output()?;

    if !output.status.success() {
        bail!("Cannot get remote.origin.url from: {}", path.display());
    }

    let url = String::from_utf8(output.stdout)?.trim().to_string();
    let mut info = parse_repo(&url, "github.com")?;
    info.clone_url = path.to_string_lossy().to_string();
    Ok(info)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ssh() {
        let info = parse_repo("git@github.com:popomore/projj.git", "github.com").unwrap();
        assert_eq!(info.host, "github.com");
        assert_eq!(info.owner, "popomore");
        assert_eq!(info.repo, "projj");
    }

    #[test]
    fn test_parse_https() {
        let info = parse_repo("https://github.com/popomore/projj.git", "github.com").unwrap();
        assert_eq!(info.host, "github.com");
        assert_eq!(info.owner, "popomore");
        assert_eq!(info.repo, "projj");
    }

    #[test]
    fn test_parse_https_no_git_suffix() {
        let info = parse_repo("https://github.com/popomore/projj", "github.com").unwrap();
        assert_eq!(info.repo, "projj");
    }

    #[test]
    fn test_parse_short_form() {
        let info = parse_repo("popomore/projj", "github.com").unwrap();
        assert_eq!(info.host, "github.com");
        assert_eq!(info.owner, "popomore");
        assert_eq!(info.repo, "projj");
        assert_eq!(info.clone_url, "git@github.com:popomore/projj.git");
    }

    #[test]
    fn test_parse_short_form_gitlab() {
        let info = parse_repo("popomore/projj", "gitlab.com").unwrap();
        assert_eq!(info.host, "gitlab.com");
        assert_eq!(info.clone_url, "git@gitlab.com:popomore/projj.git");
    }

    // #36 / #61: port should not become a directory
    #[test]
    fn test_parse_ssh_scheme_with_port() {
        let info = parse_repo("ssh://git@git.gitlab.cn:2224/web/cms.git", "github.com").unwrap();
        assert_eq!(info.host, "git.gitlab.cn");
        assert_eq!(info.owner, "web");
        assert_eq!(info.repo, "cms");
        assert_eq!(info.rel_path(), "git.gitlab.cn/web/cms");
    }

    #[test]
    fn test_parse_https_with_port() {
        let info = parse_repo("https://192.168.1.1:8080/owner/repo.git", "github.com").unwrap();
        assert_eq!(info.host, "192.168.1.1");
        assert_eq!(info.owner, "owner");
        assert_eq!(info.repo, "repo");
    }

    #[test]
    fn test_parse_https_ip_no_port() {
        let info = parse_repo("https://192.168.1.1/owner/repo.git", "github.com").unwrap();
        assert_eq!(info.host, "192.168.1.1");
        assert_eq!(info.owner, "owner");
        assert_eq!(info.repo, "repo");
    }

    #[test]
    fn test_parse_ssh_scheme_no_port() {
        let info = parse_repo("ssh://git@gitlab.com/owner/repo.git", "github.com").unwrap();
        assert_eq!(info.host, "gitlab.com");
        assert_eq!(info.owner, "owner");
        assert_eq!(info.repo, "repo");
    }

    #[test]
    fn test_parse_local_path() {
        let dir = tempfile::tempdir().unwrap();
        let repo_dir = dir.path().join("myrepo");
        std::fs::create_dir_all(&repo_dir).unwrap();

        // Init a real git repo with remote
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(&repo_dir)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args([
                "remote",
                "add",
                "origin",
                "git@github.com:testowner/testrepo.git",
            ])
            .current_dir(&repo_dir)
            .output()
            .unwrap();

        let info = parse_repo(&repo_dir.to_string_lossy(), "github.com").unwrap();
        assert_eq!(info.host, "github.com");
        assert_eq!(info.owner, "testowner");
        assert_eq!(info.repo, "testrepo");
        // clone_url should be the local path for move
        assert_eq!(
            info.clone_url,
            repo_dir.canonicalize().unwrap().to_string_lossy()
        );
    }

    #[test]
    fn test_parse_local_path_not_git() {
        let dir = tempfile::tempdir().unwrap();
        let result = parse_repo(&dir.path().to_string_lossy(), "github.com");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_local_path_nonexistent() {
        let result = parse_repo("/nonexistent/path/repo", "github.com");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_input() {
        let result = parse_repo("not-a-valid-input", "github.com");
        assert!(result.is_err());
    }

    #[test]
    fn test_move_local() {
        let src_dir = tempfile::tempdir().unwrap();
        let target_dir = tempfile::tempdir().unwrap();

        let src = src_dir.path().join("myrepo");
        std::fs::create_dir_all(src.join(".git")).unwrap();
        std::fs::write(src.join("file.txt"), "hello").unwrap();

        let target = target_dir.path().join("github.com/owner/repo");
        move_local(&src.to_string_lossy(), &target).unwrap();

        assert!(!src.exists());
        assert!(target.join(".git").exists());
        assert_eq!(
            std::fs::read_to_string(target.join("file.txt")).unwrap(),
            "hello"
        );
    }

    #[test]
    fn test_move_local_nonexistent() {
        let target_dir = tempfile::tempdir().unwrap();
        let target = target_dir.path().join("target");
        let result = move_local("/nonexistent/repo", &target);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_local_path_no_remote() {
        let dir = tempfile::tempdir().unwrap();
        let repo_dir = dir.path().join("myrepo");
        std::fs::create_dir_all(&repo_dir).unwrap();
        // Init git repo without remote
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(&repo_dir)
            .output()
            .unwrap();
        let result = parse_repo(&repo_dir.to_string_lossy(), "github.com");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_https_invalid_short() {
        // Only host, no path
        let result = parse_repo("https://github.com/onlyowner", "github.com");
        assert!(result.is_err());
    }

    #[test]
    fn test_rel_path() {
        let info = RepoInfo {
            host: "github.com".to_string(),
            owner: "popomore".to_string(),
            repo: "projj".to_string(),
            clone_url: String::new(),
        };
        assert_eq!(info.rel_path(), "github.com/popomore/projj");
    }

    #[test]
    fn test_clone_remote_progress_visible() {
        // Verify that clone_remote works end-to-end using a local bare repo.
        // The byte-streaming logic is transport-agnostic, so this covers both
        // HTTP and SSH code paths (both go through the same stderr loop).
        let tmp = tempfile::tempdir().unwrap();

        // Create a bare source repo
        let src = tmp.path().join("source.git");
        std::process::Command::new("git")
            .args(["init", "--bare", src.to_str().unwrap()])
            .output()
            .unwrap();

        // Clone it, add a commit, push back so there is something to clone
        let work = tmp.path().join("work");
        std::process::Command::new("git")
            .args(["clone", src.to_str().unwrap(), work.to_str().unwrap()])
            .output()
            .unwrap();
        std::fs::write(work.join("file.txt"), "hello").unwrap();
        std::process::Command::new("git")
            .args(["-C", work.to_str().unwrap(), "add", "."])
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args([
                "-C",
                work.to_str().unwrap(),
                "-c",
                "user.email=test@test.com",
                "-c",
                "user.name=Test",
                "commit",
                "-m",
                "init",
            ])
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["-C", work.to_str().unwrap(), "push"])
            .output()
            .unwrap();

        let target = tmp.path().join("cloned");
        clone_remote(src.to_str().unwrap(), &target).unwrap();

        assert!(target.join("file.txt").exists());
        assert_eq!(
            std::fs::read_to_string(target.join("file.txt")).unwrap(),
            "hello"
        );
    }

    #[test]
    fn test_indent_after_newline_not_carriage_return() {
        // Simulate the byte-loop logic in clone_remote to verify that \r does
        // NOT trigger a new indent prefix (git uses bare \r for in-place progress
        // overwriting), while \n does start a new indented line.
        let input = b"Receiving objects:  50%\rReceiving objects: 100%\nDone\n";
        let mut output = Vec::new();
        let mut at_line_start = true;

        for &b in input {
            if at_line_start {
                output.extend_from_slice(b"   ");
                at_line_start = false;
            }
            output.push(b);
            if b == b'\n' {
                at_line_start = true;
            }
            // NOTE: \r intentionally does NOT set at_line_start — the overwrite
            // text must follow the \r directly for terminal rendering to work.
        }

        let s = String::from_utf8(output).unwrap();
        // First line gets indent
        assert!(s.starts_with("   Receiving objects:  50%"));
        // After \r there is NO injected indent — overwrite text follows directly
        assert!(s.contains("\rReceiving objects: 100%\n"));
        // After \n the next line is indented
        assert!(s.contains("\n   Done\n"));
    }
}
