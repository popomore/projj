use std::path::{Path, PathBuf};

use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Repo {
    pub path: PathBuf,
    pub base: PathBuf,
    pub host: String,
    pub owner: String,
    pub name: String,
}

impl Repo {
    /// owner/repo
    pub fn short_key(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }

    /// host/owner/repo
    pub fn display_key(&self) -> String {
        format!("{}/{}/{}", self.host, self.owner, self.name)
    }

    /// Construct git URL from path components
    pub fn git_url(&self) -> String {
        format!("git@{}:{}/{}.git", self.host, self.owner, self.name)
    }
}

/// Scan base directories for git repositories.
/// Fixed depth: base/host/owner/repo/.git
pub fn scan(base_dirs: &[PathBuf]) -> Result<Vec<Repo>> {
    let mut repos = Vec::new();
    for base in base_dirs {
        if !base.exists() {
            continue;
        }
        scan_base(base, &mut repos)?;
    }
    repos.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(repos)
}

fn scan_base(base: &Path, repos: &mut Vec<Repo>) -> Result<()> {
    let base_path = base.to_path_buf();
    // Level 1: host (github.com, gitlab.com, ...)
    let Ok(hosts) = std::fs::read_dir(base) else {
        return Ok(());
    };
    for host_entry in hosts {
        let host_entry = host_entry?;
        if !host_entry.file_type()?.is_dir() {
            continue;
        }
        let host_name = host_entry.file_name().to_string_lossy().to_string();
        if host_name.starts_with('.') {
            continue;
        }

        // Level 2: owner
        let Ok(owners) = std::fs::read_dir(host_entry.path()) else {
            continue;
        };
        for owner_entry in owners {
            let owner_entry = owner_entry?;
            if !owner_entry.file_type()?.is_dir() {
                continue;
            }
            let owner_name = owner_entry.file_name().to_string_lossy().to_string();
            if owner_name.starts_with('.') {
                continue;
            }

            // Level 3: repo
            let Ok(repo_entries) = std::fs::read_dir(owner_entry.path()) else {
                continue;
            };
            for repo_entry in repo_entries {
                let repo_entry = repo_entry?;
                if !repo_entry.file_type()?.is_dir() {
                    continue;
                }
                let repo_name = repo_entry.file_name().to_string_lossy().to_string();
                if repo_name.starts_with('.') {
                    continue;
                }

                if repo_entry.path().join(".git").exists() {
                    repos.push(Repo {
                        path: repo_entry.path(),
                        base: base_path.clone(),
                        host: host_name.clone(),
                        owner: owner_name.clone(),
                        name: repo_name,
                    });
                }
            }
        }
    }
    Ok(())
}

/// Find repos matching a keyword (case-insensitive).
///
/// Returns all matches. Exact matches (ends with /keyword) are sorted first,
/// followed by partial matches (contains keyword).
pub fn find(repos: &[Repo], keyword: &str) -> Vec<Repo> {
    let kw_lower = keyword.to_lowercase();
    let keyword_suffix = format!("/{}", kw_lower.trim_start_matches('/'));

    let mut exact = Vec::new();
    let mut partial = Vec::new();

    for repo in repos {
        let key = repo.display_key().to_lowercase();
        if key.ends_with(&keyword_suffix) {
            exact.push(repo.clone());
        } else if key.contains(&kw_lower) {
            partial.push(repo.clone());
        }
    }

    exact.extend(partial);
    exact
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_repo(host: &str, owner: &str, name: &str) -> Repo {
        Repo {
            path: PathBuf::from(format!("/base/{host}/{owner}/{name}")),
            base: PathBuf::from("/base"),
            host: host.to_string(),
            owner: owner.to_string(),
            name: name.to_string(),
        }
    }

    #[test]
    fn test_repo_display_key() {
        let repo = make_repo("github.com", "popomore", "projj");
        assert_eq!(repo.display_key(), "github.com/popomore/projj");
    }

    #[test]
    fn test_repo_short_key() {
        let repo = make_repo("github.com", "popomore", "projj");
        assert_eq!(repo.short_key(), "popomore/projj");
    }

    #[test]
    fn test_repo_git_url() {
        let repo = make_repo("github.com", "popomore", "projj");
        assert_eq!(repo.git_url(), "git@github.com:popomore/projj.git");
    }

    #[test]
    fn test_find_exact_match() {
        let repos = vec![
            make_repo("github.com", "popomore", "projj"),
            make_repo("github.com", "popomore", "tiny-projj"),
        ];
        let result = find(&repos, "projj");
        // Both match: "projj" exact first, "tiny-projj" partial second
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "projj");
        assert_eq!(result[1].name, "tiny-projj");
    }

    #[test]
    fn test_find_case_insensitive() {
        let repos = vec![make_repo("github.com", "popomore", "Projj")];
        let result = find(&repos, "projj");
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_find_owner_repo() {
        let repos = vec![
            make_repo("github.com", "popomore", "projj"),
            make_repo("github.com", "other", "projj"),
        ];
        let result = find(&repos, "popomore/projj");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].owner, "popomore");
    }

    #[test]
    fn test_find_no_match() {
        let repos = vec![make_repo("github.com", "popomore", "projj")];
        let result = find(&repos, "nonexistent");
        assert!(result.is_empty());
    }

    #[test]
    fn test_find_partial_match() {
        let repos = vec![
            make_repo("github.com", "popomore", "projj"),
            make_repo("github.com", "SeeleAI", "projj-tools"),
        ];
        let result = find(&repos, "projj");
        assert_eq!(result.len(), 2);
        // Exact match first
        assert_eq!(result[0].name, "projj");
        assert_eq!(result[1].name, "projj-tools");
    }

    #[test]
    fn test_scan_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        let repos = scan(&[dir.path().to_path_buf()]).unwrap();
        assert!(repos.is_empty());
    }

    #[test]
    fn test_scan_with_repos() {
        let dir = tempfile::tempdir().unwrap();
        let repo_path = dir.path().join("github.com/popomore/projj/.git");
        std::fs::create_dir_all(&repo_path).unwrap();
        let repos = scan(&[dir.path().to_path_buf()]).unwrap();
        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].host, "github.com");
        assert_eq!(repos[0].owner, "popomore");
        assert_eq!(repos[0].name, "projj");
    }

    #[test]
    fn test_scan_skips_hidden_dirs() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".hidden/owner/repo/.git")).unwrap();
        std::fs::create_dir_all(dir.path().join("github.com/.hidden/repo/.git")).unwrap();
        std::fs::create_dir_all(dir.path().join("github.com/owner/.hidden/.git")).unwrap();
        let repos = scan(&[dir.path().to_path_buf()]).unwrap();
        assert!(repos.is_empty());
    }

    #[test]
    fn test_scan_skips_non_git_dirs() {
        let dir = tempfile::tempdir().unwrap();
        // No .git directory
        std::fs::create_dir_all(dir.path().join("github.com/owner/repo")).unwrap();
        let repos = scan(&[dir.path().to_path_buf()]).unwrap();
        assert!(repos.is_empty());
    }

    #[test]
    fn test_scan_nonexistent_base() {
        let repos = scan(&[PathBuf::from("/nonexistent/path")]).unwrap();
        assert!(repos.is_empty());
    }

    #[test]
    fn test_scan_multiple_bases() {
        let dir1 = tempfile::tempdir().unwrap();
        let dir2 = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir1.path().join("github.com/a/repo1/.git")).unwrap();
        std::fs::create_dir_all(dir2.path().join("gitlab.com/b/repo2/.git")).unwrap();
        let repos = scan(&[dir1.path().to_path_buf(), dir2.path().to_path_buf()]).unwrap();
        assert_eq!(repos.len(), 2);
    }
}
