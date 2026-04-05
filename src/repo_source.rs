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
    let hosts = match std::fs::read_dir(base) {
        Ok(entries) => entries,
        Err(_) => return Ok(()),
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
        let owners = match std::fs::read_dir(host_entry.path()) {
            Ok(entries) => entries,
            Err(_) => continue,
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
            let repo_entries = match std::fs::read_dir(owner_entry.path()) {
                Ok(entries) => entries,
                Err(_) => continue,
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
