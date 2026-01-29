use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::error::{AppError, Result};

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
        let entry = keyring::Entry::new(&self.service, "tokens")
            .map_err(|e| AppError::Keyring(format!("Failed to create entry: {e}")))?;

        let json = serde_json::to_string(tokens)
            .map_err(|e| AppError::Keyring(format!("Failed to serialize tokens: {e}")))?;

        entry
            .set_password(&json)
            .map_err(|e| AppError::Keyring(format!("Failed to set password: {e}")))?;

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
        if !self.path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&self.path)?;
        let tokens: TokenPair = serde_json::from_str(&content)?;
        Ok(Some(tokens))
    }

    fn save(&self, tokens: &TokenPair) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(tokens)?;
        std::fs::write(&self.path, json)?;
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

/// Returns a token store, preferring keyring but falling back to file
pub fn get_token_store() -> Box<dyn TokenStore> {
    let keyring = KeyringStore::new("term-workspace");

    // Try to use keyring first
    if keyring.load().is_ok() {
        return Box::new(keyring);
    }

    // Try to create a test entry to see if keyring works
    let test_entry = keyring::Entry::new("term-workspace-test", "test");
    if let Ok(entry) = test_entry {
        if entry.set_password("test").is_ok() {
            let _ = entry.delete_credential();
            return Box::new(keyring);
        }
    }

    // Fall back to file store
    Box::new(FileStore::default())
}
