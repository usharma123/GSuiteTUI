use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("OAuth error: {0}")]
    OAuth(String),

    #[error("API error (status {status}): {message}")]
    Api { status: u16, message: String },

    #[error("Token expired")]
    TokenExpired,

    #[error("Sync token gone (410)")]
    SyncTokenGone,

    #[error("Keyring error: {0}")]
    Keyring(String),

    #[error("MIME error: {0}")]
    Mime(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("{0}")]
    Other(String),
}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        AppError::Network(e.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Parse(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
