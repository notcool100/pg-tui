use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionProfile {
    pub name: String,
    pub host: String,
    pub port: String,
    pub database: String,
    pub user: String,
    // Note: password is not saved for security reasons
}

impl ConnectionProfile {
    pub fn new(name: String) -> Self {
        Self {
            name,
            host: "localhost".to_string(),
            port: "5432".to_string(),
            database: "postgres".to_string(),
            user: "postgres".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub connections: Vec<ConnectionProfile>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if !config_path.exists() {
            return Ok(Self::default());
        }

        let contents = std::fs::read_to_string(config_path)?;
        let config: Config = serde_json::from_str(&contents)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let contents = serde_json::to_string_pretty(self)?;
        std::fs::write(config_path, contents)?;
        Ok(())
    }

    fn config_path() -> Result<PathBuf> {
        let mut path = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        path.push("psql_cli");
        path.push("config.json");
        Ok(path)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            connections: vec![],
        }
    }
}
