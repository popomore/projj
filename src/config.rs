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
    /// 1. [scripts] table in config
    /// 2. Executable file in ~/.projj/hooks/
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
            if s.starts_with('~') {
                home.join(&s[2..]).to_string_lossy().to_string()
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
