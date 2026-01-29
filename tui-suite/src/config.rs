use std::path::PathBuf;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub google_client_id: Option<String>,
    pub google_client_secret: Option<String>,
    pub calendar_id: String,
    pub notes_path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            google_client_id: None,
            google_client_secret: None,
            calendar_id: "primary".to_string(),
            notes_path: PathBuf::from("notes.md"),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)
                .map_err(|e| AppError::Config(format!("Failed to read config: {e}")))?;
            let config: Config = serde_json::from_str(&content)
                .map_err(|e| AppError::Config(format!("Failed to parse config: {e}")))?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AppError::Config(format!("Failed to create config dir: {e}")))?;
        }
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| AppError::Config(format!("Failed to serialize config: {e}")))?;
        std::fs::write(&config_path, content)
            .map_err(|e| AppError::Config(format!("Failed to write config: {e}")))?;
        Ok(())
    }

    pub fn config_dir() -> Result<PathBuf> {
        ProjectDirs::from("", "", "term-workspace")
            .map(|p| p.config_dir().to_path_buf())
            .ok_or_else(|| AppError::Config("Could not determine config directory".to_string()))
    }

    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.json"))
    }

    pub fn cache_dir() -> Result<PathBuf> {
        ProjectDirs::from("", "", "term-workspace")
            .map(|p| p.cache_dir().to_path_buf())
            .ok_or_else(|| AppError::Config("Could not determine cache directory".to_string()))
    }
}
