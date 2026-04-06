use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub base: BaseDir,
    #[serde(default = "default_platform")]
    pub platform: String,
    #[serde(default)]
    pub scripts: HashMap<String, String>,
    #[serde(default)]
    pub hooks: Vec<HookEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookEntry {
    pub event: String,
    #[serde(default)]
    pub matcher: Option<String>,
    pub command: String,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BaseDir {
    Single(String),
    Multiple(Vec<String>),
}

fn default_platform() -> String {
    "github.com".to_string()
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = config_path();
        if !path.exists() {
            bail!("Configuration not found. Please run `projj init` first.");
        }
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config: {}", path.display()))?;
        let mut config: Config =
            toml::from_str(&content).with_context(|| "Failed to parse config.toml")?;
        config.resolve_paths();
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    pub fn base_dirs(&self) -> Vec<PathBuf> {
        match &self.base {
            BaseDir::Single(s) => vec![PathBuf::from(s)],
            BaseDir::Multiple(v) => v.iter().map(PathBuf::from).collect(),
        }
    }

    /// Resolve a script name with three-level lookup:
    /// 1. `scripts` table in config
    /// 2. Executable file in `~/.projj/hooks/`
    /// 3. Raw command as-is
    pub fn resolve_script(&self, name: &str) -> String {
        // 1. Scripts table
        if let Some(script) = self.scripts.get(name) {
            return script.clone();
        }

        // 2. ~/.projj/hooks/ directory (only for simple names without spaces/slashes)
        if !name.contains(' ') && !name.contains('/') {
            let hook_path = hooks_dir().join(name);
            if hook_path.exists() {
                return hook_path.to_string_lossy().to_string();
            }
        }

        // 3. Raw command
        name.to_string()
    }

    fn resolve_paths(&mut self) {
        let home = dirs::home_dir().unwrap_or_default();
        let resolve = |s: &str| -> String {
            if s == "~" {
                home.to_string_lossy().to_string()
            } else if let Some(rest) = s.strip_prefix("~/") {
                home.join(rest).to_string_lossy().to_string()
            } else if s.starts_with('.') {
                config_dir().join(s).to_string_lossy().to_string()
            } else {
                s.to_string()
            }
        };
        match &mut self.base {
            BaseDir::Single(s) => *s = resolve(s),
            BaseDir::Multiple(v) => {
                for s in v.iter_mut() {
                    *s = resolve(s);
                }
            }
        }
    }
}

pub fn config_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".projj")
}

pub fn hooks_dir() -> PathBuf {
    config_dir().join("hooks")
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.toml")
}

pub fn config_exists() -> bool {
    config_path().exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_config() {
        let toml = r#"base = "/tmp/projj""#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.platform, "github.com");
        assert!(config.scripts.is_empty());
        assert!(config.hooks.is_empty());
    }

    #[test]
    fn test_parse_full_config() {
        let toml = r#"
base = ["/tmp/a", "/tmp/b"]
platform = "gitlab.com"

[scripts]
clean = "rm -rf node_modules"

[[hooks]]
event = "post_add"
matcher = "github\\.com"
command = "clean"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.platform, "gitlab.com");
        assert_eq!(config.base_dirs().len(), 2);
        assert_eq!(config.scripts["clean"], "rm -rf node_modules");
        assert_eq!(config.hooks.len(), 1);
        assert_eq!(config.hooks[0].event, "post_add");
        assert_eq!(config.hooks[0].matcher.as_deref(), Some("github\\.com"));
    }

    #[test]
    fn test_base_dirs_single() {
        let config = Config {
            base: BaseDir::Single("/tmp/projj".to_string()),
            platform: "github.com".to_string(),
            scripts: HashMap::new(),
            hooks: vec![],
        };
        assert_eq!(config.base_dirs(), vec![PathBuf::from("/tmp/projj")]);
    }

    #[test]
    fn test_base_dirs_multiple() {
        let config = Config {
            base: BaseDir::Multiple(vec!["/tmp/a".to_string(), "/tmp/b".to_string()]),
            platform: "github.com".to_string(),
            scripts: HashMap::new(),
            hooks: vec![],
        };
        assert_eq!(config.base_dirs().len(), 2);
    }

    #[test]
    fn test_resolve_script_from_table() {
        let mut scripts = HashMap::new();
        scripts.insert("clean".to_string(), "rm -rf node_modules".to_string());
        let config = Config {
            base: BaseDir::Single("/tmp".to_string()),
            platform: "github.com".to_string(),
            scripts,
            hooks: vec![],
        };
        assert_eq!(config.resolve_script("clean"), "rm -rf node_modules");
    }

    #[test]
    fn test_resolve_script_raw_command() {
        let config = Config {
            base: BaseDir::Single("/tmp".to_string()),
            platform: "github.com".to_string(),
            scripts: HashMap::new(),
            hooks: vec![],
        };
        assert_eq!(config.resolve_script("git status"), "git status");
    }

    #[test]
    fn test_resolve_paths_tilde() {
        let toml = r#"base = "~/projj""#;
        let mut config: Config = toml::from_str(toml).unwrap();
        config.resolve_paths();
        let home = dirs::home_dir().unwrap();
        assert_eq!(config.base_dirs(), vec![home.join("projj")]);
    }

    #[test]
    fn test_resolve_paths_bare_tilde() {
        let toml = r#"base = "~""#;
        let mut config: Config = toml::from_str(toml).unwrap();
        config.resolve_paths();
        let home = dirs::home_dir().unwrap();
        assert_eq!(config.base_dirs(), vec![home]);
    }

    #[test]
    fn test_resolve_paths_absolute() {
        let toml = r#"base = "/absolute/path""#;
        let mut config: Config = toml::from_str(toml).unwrap();
        config.resolve_paths();
        assert_eq!(config.base_dirs(), vec![PathBuf::from("/absolute/path")]);
    }

    #[test]
    fn test_hook_entry_with_env() {
        let toml = r#"
base = "/tmp"

[[hooks]]
event = "post_add"
command = "test"
env = { NAME = "value" }
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.hooks[0].env["NAME"], "value");
    }
}
