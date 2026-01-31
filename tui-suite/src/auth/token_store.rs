use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::io::Write;

use crate::config::Config;
use crate::error::{AppError, Result};

fn debug_log(msg: &str) {
    use std::fs::OpenOptions;
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/tui-debug.log")
    {
        let timestamp = chrono::Local::now().format("%H:%M:%S%.3f");
        let _ = writeln!(file, "[{}] token_store: {}", timestamp, msg);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: DateTime<Utc>,
}

impl TokenPair {
    pub fn is_expired(&self) -> bool {
        // Consider expired if less than 5 minutes remaining
        self.expires_at < Utc::now() + chrono::Duration::minutes(5)
    }
}

pub trait TokenStore: Send + Sync {
    fn load(&self) -> Result<Option<TokenPair>>;
    fn save(&self, tokens: &TokenPair) -> Result<()>;
    fn clear(&self) -> Result<()>;
}

pub struct KeyringStore {
    service: String,
}

impl KeyringStore {
    pub fn new(service: &str) -> Self {
        Self {
            service: service.to_string(),
        }
    }
}

impl TokenStore for KeyringStore {
    fn load(&self) -> Result<Option<TokenPair>> {
        let entry = keyring::Entry::new(&self.service, "tokens")
            .map_err(|e| AppError::Keyring(format!("Failed to create entry: {e}")))?;

        match entry.get_password() {
            Ok(json) => {
                let tokens: TokenPair = serde_json::from_str(&json)
                    .map_err(|e| AppError::Keyring(format!("Failed to parse tokens: {e}")))?;
                Ok(Some(tokens))
            }
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(AppError::Keyring(format!("Failed to get password: {e}"))),
        }
    }

    fn save(&self, tokens: &TokenPair) -> Result<()> {
        debug_log(&format!("KeyringStore::save called, service: {}", self.service));
        let entry = keyring::Entry::new(&self.service, "tokens")
            .map_err(|e| AppError::Keyring(format!("Failed to create entry: {e}")))?;

        let json = serde_json::to_string(tokens)
            .map_err(|e| AppError::Keyring(format!("Failed to serialize tokens: {e}")))?;

        entry
            .set_password(&json)
            .map_err(|e| AppError::Keyring(format!("Failed to set password: {e}")))?;

        debug_log("KeyringStore::save succeeded");
        Ok(())
    }

    fn clear(&self) -> Result<()> {
        let entry = keyring::Entry::new(&self.service, "tokens")
            .map_err(|e| AppError::Keyring(format!("Failed to create entry: {e}")))?;

        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(AppError::Keyring(format!("Failed to delete: {e}"))),
        }
    }
}

pub struct FileStore {
    path: std::path::PathBuf,
}

impl FileStore {
    pub fn new() -> Result<Self> {
        let path = Config::config_dir()?.join("tokens.json");
        Ok(Self { path })
    }
}

impl TokenStore for FileStore {
    fn load(&self) -> Result<Option<TokenPair>> {
        debug_log(&format!("FileStore::load, path: {:?}", self.path));
        if !self.path.exists() {
            debug_log("FileStore: path does not exist");
            return Ok(None);
        }

        let content = std::fs::read_to_string(&self.path)?;
        let tokens: TokenPair = serde_json::from_str(&content)?;
        debug_log("FileStore::load succeeded");
        Ok(Some(tokens))
    }

    fn save(&self, tokens: &TokenPair) -> Result<()> {
        debug_log(&format!("FileStore::save, path: {:?}", self.path));
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(tokens)?;
        std::fs::write(&self.path, json)?;
        debug_log("FileStore::save succeeded");
        Ok(())
    }

    fn clear(&self) -> Result<()> {
        if self.path.exists() {
            std::fs::remove_file(&self.path)?;
        }
        Ok(())
    }
}

impl Default for FileStore {
    fn default() -> Self {
        Self::new().expect("Failed to create FileStore")
    }
}

/// Returns a token store - using FileStore for reliability
/// (macOS keyring has inconsistent behavior with unsigned apps)
pub fn get_token_store() -> Box<dyn TokenStore> {
    debug_log("Using FileStore for token storage");
    Box::new(FileStore::default())
}
