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
    pub tasks: HashMap<String, String>,
    #[serde(default)]
    pub hooks: Vec<HookEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookEntry {
    pub event: String,
    #[serde(default)]
    pub matcher: Option<String>,
    /// Tasks to execute in order. Each can be a task name or raw command.
    #[serde(default)]
    pub tasks: Vec<String>,
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
        Self::load_from(&config_path())
    }

    pub fn load_from(path: &PathBuf) -> Result<Self> {
        if !path.exists() {
            bail!("Configuration not found. Please run `projj init` first.");
        }
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config: {}", path.display()))?;
        let mut config: Config =
            toml::from_str(&content).with_context(|| "Failed to parse config.toml")?;
        config.resolve_paths();
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        self.save_to(&config_path())
    }

    pub fn save_to(&self, path: &PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn base_dirs(&self) -> Vec<PathBuf> {
        match &self.base {
            BaseDir::Single(s) => vec![PathBuf::from(s)],
            BaseDir::Multiple(v) => v.iter().map(PathBuf::from).collect(),
        }
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
    // PROJJ_HOME overrides home directory detection (useful for testing)
    if let Ok(home) = std::env::var("PROJJ_HOME") {
        return PathBuf::from(home).join(".projj");
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".projj")
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
        assert!(config.tasks.is_empty());
        assert!(config.hooks.is_empty());
    }

    #[test]
    fn test_parse_full_config() {
        let toml = r#"
base = ["/tmp/a", "/tmp/b"]
platform = "gitlab.com"

[tasks]
clean = "rm -rf node_modules"

[[hooks]]
event = "post_add"
matcher = "github\\.com"
tasks = ["clean"]
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.platform, "gitlab.com");
        assert_eq!(config.base_dirs().len(), 2);
        assert_eq!(config.tasks["clean"], "rm -rf node_modules");
        assert_eq!(config.hooks.len(), 1);
        assert_eq!(config.hooks[0].event, "post_add");
        assert_eq!(config.hooks[0].matcher.as_deref(), Some("github\\.com"));
    }

    #[test]
    fn test_base_dirs_single() {
        let config = Config {
            base: BaseDir::Single("/tmp/projj".to_string()),
            platform: "github.com".to_string(),
            tasks: HashMap::new(),
            hooks: vec![],
        };
        assert_eq!(config.base_dirs(), vec![PathBuf::from("/tmp/projj")]);
    }

    #[test]
    fn test_base_dirs_multiple() {
        let config = Config {
            base: BaseDir::Multiple(vec!["/tmp/a".to_string(), "/tmp/b".to_string()]),
            platform: "github.com".to_string(),
            tasks: HashMap::new(),
            hooks: vec![],
        };
        assert_eq!(config.base_dirs().len(), 2);
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
    fn test_resolve_paths_relative() {
        let toml = r#"base = "./data""#;
        let mut config: Config = toml::from_str(toml).unwrap();
        config.resolve_paths();
        let expected = config_dir().join("./data");
        assert_eq!(config.base_dirs(), vec![expected]);
    }

    #[test]
    fn test_resolve_paths_multiple() {
        let toml = r#"base = ["~/a", "/b"]"#;
        let mut config: Config = toml::from_str(toml).unwrap();
        config.resolve_paths();
        let home = dirs::home_dir().unwrap();
        assert_eq!(config.base_dirs()[0], home.join("a"));
        assert_eq!(config.base_dirs()[1], PathBuf::from("/b"));
    }

    #[test]
    fn test_config_dir_and_paths() {
        let dir = config_dir();
        assert!(dir.ends_with(".projj"));
        assert!(config_path().ends_with("config.toml"));
    }

    #[test]
    fn test_config_exists_false() {
        // config_exists depends on real filesystem, just verify it returns bool
        let _ = config_exists();
    }

    #[test]
    fn test_save_and_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let config = Config {
            base: BaseDir::Single("/tmp/projj".to_string()),
            platform: "github.com".to_string(),
            tasks: HashMap::new(),
            hooks: vec![HookEntry {
                event: "post_add".to_string(),
                matcher: None,
                tasks: vec!["echo hi".to_string()],
                env: HashMap::new(),
            }],
        };
        config.save_to(&path).unwrap();
        assert!(path.exists());

        let loaded = Config::load_from(&path).unwrap();
        assert_eq!(loaded.platform, "github.com");
        assert_eq!(loaded.hooks.len(), 1);
    }

    #[test]
    fn test_load_nonexistent() {
        let path = PathBuf::from("/nonexistent/config.toml");
        let result = Config::load_from(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_hook_entry_with_env() {
        let toml = r#"
base = "/tmp"

[[hooks]]
event = "post_add"
tasks = ["test"]
env = { NAME = "value" }
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.hooks[0].env["NAME"], "value");
    }
}
