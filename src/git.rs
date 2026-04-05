use std::path::Path;

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

/// Parse a repo argument into RepoInfo.
///
/// Supported formats:
/// - git@github.com:owner/repo.git                (SSH)
/// - ssh://git@git.gitlab.cn:2224/owner/repo.git  (SSH with port)
/// - https://github.com/owner/repo.git            (HTTPS)
/// - https://192.168.1.1:8080/owner/repo.git      (HTTPS with IP:port)
/// - owner/repo                                   (short form)
/// - ./path/to/local/repo                         (local path)
pub fn parse_repo(input: &str, default_platform: &str) -> Result<RepoInfo> {
    // Local path
    if input.starts_with('.') || input.starts_with('/') {
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
        let clone_url = format!("git@{}:{}/{}.git", host, owner, repo);
        return Ok(RepoInfo {
            host,
            owner,
            repo,
            clone_url,
        });
    }

    bail!("Cannot parse repository: {}", input)
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
        .ok_or_else(|| anyhow::anyhow!("Invalid SSH URL: {}", input))?;
    let path = path.strip_suffix(".git").unwrap_or(path);
    let (owner, repo) = path
        .split_once('/')
        .ok_or_else(|| anyhow::anyhow!("Invalid SSH URL: {}", input))?;
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
        .ok_or_else(|| anyhow::anyhow!("Invalid SSH URL: {}", input))?;

    // Split host:port from path
    // Could be: git.gitlab.cn:2224/web/cms.git
    // or:       git.gitlab.cn/web/cms.git
    let (host_part, path) = split_host_path(without_scheme)
        .ok_or_else(|| anyhow::anyhow!("Invalid SSH URL: {}", input))?;

    let path = path.strip_suffix(".git").unwrap_or(path);
    let (owner, repo) = path
        .split_once('/')
        .ok_or_else(|| anyhow::anyhow!("Invalid SSH URL: {}", input))?;

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
        .ok_or_else(|| anyhow::anyhow!("Invalid HTTPS URL: {}", input))?;

    let parts: Vec<&str> = path.splitn(2, '/').collect();
    if parts.len() < 2 {
        bail!("Invalid HTTPS URL: {}", input);
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

/// Split "host[:port]/rest/of/path" into ("host[:port]", "rest/of/path")
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
        .map_err(|_| anyhow::anyhow!("Local path does not exist: {}", input))?;

    let git_dir = path.join(".git");
    if !git_dir.exists() {
        bail!("Not a git repository: {}", input);
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
}
