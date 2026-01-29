use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::calendar::CalendarEvent;
use crate::config::Config;
use crate::error::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct EventCache {
    pub events: Vec<CalendarEvent>,
    pub sync_token: Option<String>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl EventCache {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            sync_token: None,
            last_updated: chrono::Utc::now(),
        }
    }

    pub fn cache_path() -> Result<PathBuf> {
        Ok(Config::cache_dir()?.join("events.json"))
    }

    pub fn load() -> Result<Option<Self>> {
        let path = Self::cache_path()?;
        if !path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&path)?;
        let cache: Self = serde_json::from_str(&content)?;
        Ok(Some(cache))
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::cache_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    pub fn clear() -> Result<()> {
        let path = Self::cache_path()?;
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }

    pub fn update(&mut self, events: Vec<CalendarEvent>, sync_token: Option<String>) {
        self.events = events;
        self.sync_token = sync_token;
        self.last_updated = chrono::Utc::now();
    }
}

impl Default for EventCache {
    fn default() -> Self {
        Self::new()
    }
}
