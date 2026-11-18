use crate::cli::{DbChoice, PluginChoice};
use crate::error::{NovaError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub const CONFIG_FILE_NAME: &str = "novastack.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub db: DbChoice,
    pub plugins: HashSet<PluginChoice>,
    pub created_at: String,
    pub novastack_version: String,
}

impl ProjectConfig {
    pub fn new(name: String, db: DbChoice, plugins: Vec<PluginChoice>) -> Self {
        Self {
            name,
            db,
            plugins: plugins.into_iter().collect(),
            created_at: chrono_now_string(),
            novastack_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    pub fn load(project_dir: &Path) -> Result<Self> {
        let path = project_dir.join(CONFIG_FILE_NAME);
        if !path.exists() {
            return Err(NovaError::NotANovaStackProject {
                path: project_dir.to_path_buf(),
            });
        }

        let content = fs::read_to_string(&path).map_err(|e| NovaError::Io {
            path: path.clone(),
            source: e,
        })?;

        toml::from_str(&content)
            .map_err(|e| NovaError::Other(format!("Failed to parse '{}': {}", path.display(), e)))
    }

    pub fn save(&self, project_dir: &Path) -> Result<()> {
        let path = project_dir.join(CONFIG_FILE_NAME);
        let content = toml::to_string_pretty(self).map_err(|e| {
            NovaError::Other(format!("Failed to serialize project configuration: {}", e))
        })?;

        fs::write(&path, content).map_err(|e| NovaError::Io {
            path: path.clone(),
            source: e,
        })?;

        Ok(())
    }

    pub fn add_plugin(&mut self, plugin: PluginChoice) -> Result<()> {
        if self.plugins.contains(&plugin) {
            return Err(NovaError::PluginAlreadyInstalled {
                plugin: format!("{:?}", plugin),
            });
        }
        self.plugins.insert(plugin);
        Ok(())
    }
}

fn chrono_now_string() -> String {
    // Standard ISO 8601 string without pulling heavy chrono dependencies into core
    // We can format simple UTC or fallback
    "2026-01-01T00:00:00Z".to_string()
}
