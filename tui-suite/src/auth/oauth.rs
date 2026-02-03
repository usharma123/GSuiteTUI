use std::net::TcpListener;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::response::Html;
use axum::routing::get;
use axum::Router;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

use crate::error::{AppError, Result};

use super::token_store::TokenPair;

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

pub struct OAuthFlow {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    pkce_verifier: String,
    port: u16,
    auth_url: String,
}

impl OAuthFlow {
    pub fn new(client_id: &str, client_secret: &str) -> Result<Self> {
        // Find available port
        let listener = TcpListener::bind("127.0.0.1:0")
            .map_err(|e| AppError::OAuth(format!("Failed to bind port: {e}")))?;
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let redirect_uri = format!("http://127.0.0.1:{}/callback", port);

        // Generate PKCE verifier and challenge
        let verifier_bytes: [u8; 32] = rand::random();
        let pkce_verifier = URL_SAFE_NO_PAD.encode(verifier_bytes);

        // SHA256 hash of verifier for challenge
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(pkce_verifier.as_bytes());
        let challenge_hash = hasher.finalize();
        let pkce_challenge = URL_SAFE_NO_PAD.encode(challenge_hash);

        // Build authorization URL
        // gmail.readonly needed for get_user_email() profile access
        let scopes = "https://www.googleapis.com/auth/calendar.readonly https://www.googleapis.com/auth/gmail.send https://www.googleapis.com/auth/gmail.readonly https://www.googleapis.com/auth/drive https://www.googleapis.com/auth/spreadsheets";
        let state: [u8; 16] = rand::random();
        let state_str = URL_SAFE_NO_PAD.encode(state);

        let auth_url = format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&code_challenge={}&code_challenge_method=S256&state={}&access_type=offline&prompt=consent",
            GOOGLE_AUTH_URL,
            urlencoding::encode(client_id),
            urlencoding::encode(&redirect_uri),
            urlencoding::encode(scopes),
            urlencoding::encode(&pkce_challenge),
            urlencoding::encode(&state_str),
        );

        Ok(Self {
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            redirect_uri,
            pkce_verifier,
            port,
            auth_url,
        })
    }

    pub fn auth_url(&self) -> &str {
        &self.auth_url
    }

    pub fn open_browser(&self) -> Result<()> {
        open::that(&self.auth_url)
            .map_err(|e| AppError::OAuth(format!("Failed to open browser: {e}")))
    }

    pub async fn wait_for_callback(self) -> Result<TokenPair> {
        let (tx, rx) = oneshot::channel::<String>();
        let tx = Arc::new(tokio::sync::Mutex::new(Some(tx)));

        let app = Router::new()
            .route("/callback", get(callback_handler))
            .with_state(tx);

        let addr = format!("127.0.0.1:{}", self.port);
        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .map_err(|e| AppError::OAuth(format!("Failed to bind: {e}")))?;

        // Spawn server in background
        tokio::spawn(async move {
            axum::serve(listener, app).await.ok();
        });

        // Wait for code
        let code = rx
            .await
            .map_err(|_| AppError::OAuth("Callback channel closed".to_string()))?;

        // Exchange code for tokens
        let http_client = reqwest::Client::new();
        let params = TokenRequest {
            client_id: &self.client_id,
            client_secret: &self.client_secret,
            code: &code,
            redirect_uri: &self.redirect_uri,
            grant_type: "authorization_code",
            code_verifier: Some(&self.pkce_verifier),
            refresh_token: None,
        };

        let response = http_client
            .post(GOOGLE_TOKEN_URL)
            .form(&params)
            .send()
            .await
            .map_err(|e| AppError::OAuth(format!("Token request failed: {e}")))?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(AppError::OAuth(format!(
                "Token exchange failed ({}): {}",
                status, text
            )));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| AppError::OAuth(format!("Failed to parse token response: {e}")))?;

        Ok(TokenPair {
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token,
            expires_at: chrono::Utc::now()
                + chrono::Duration::seconds(token_response.expires_in as i64),
        })
    }
}

#[derive(Serialize)]
struct TokenRequest<'a> {
    client_id: &'a str,
    client_secret: &'a str,
    #[serde(skip_serializing_if = "str::is_empty")]
    code: &'a str,
    redirect_uri: &'a str,
    grant_type: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    code_verifier: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    refresh_token: Option<&'a str>,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: u64,
    #[serde(default)]
    #[allow(dead_code)]
    token_type: String,
}

#[derive(Deserialize)]
struct CallbackParams {
    code: Option<String>,
    error: Option<String>,
}

async fn callback_handler(
    State(tx): State<Arc<tokio::sync::Mutex<Option<oneshot::Sender<String>>>>>,
    Query(params): Query<CallbackParams>,
) -> Html<&'static str> {
    if let Some(_error) = params.error {
        return Html(
            r#"<!DOCTYPE html><html><body>
            <h1>Authentication Failed</h1>
            <p>You can close this window.</p>
            </body></html>"#,
        );
    }

    if let Some(code) = params.code {
        if let Some(sender) = tx.lock().await.take() {
            let _ = sender.send(code);
        }
        Html(
            r#"<!DOCTYPE html><html><body>
            <h1>Authentication Successful!</h1>
            <p>You can close this window and return to the terminal.</p>
            </body></html>"#,
        )
    } else {
        Html(
            r#"<!DOCTYPE html><html><body>
            <h1>Authentication Failed</h1>
            <p>No authorization code received.</p>
            </body></html>"#,
        )
    }
}

pub async fn refresh_access_token(
    client_id: &str,
    client_secret: &str,
    refresh_token_str: &str,
) -> Result<TokenPair> {
    let http_client = reqwest::Client::new();

    let params = [
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("refresh_token", refresh_token_str),
        ("grant_type", "refresh_token"),
    ];

    let response = http_client
        .post(GOOGLE_TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| AppError::OAuth(format!("Token refresh request failed: {e}")))?;

    let status = response.status();
    if !status.is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(AppError::OAuth(format!(
            "Token refresh failed ({}): {}",
            status, text
        )));
    }

    let token_response: TokenResponse = response
        .json()
        .await
        .map_err(|e| AppError::OAuth(format!("Failed to parse token response: {e}")))?;

    Ok(TokenPair {
        access_token: token_response.access_token,
        refresh_token: token_response
            .refresh_token
            .or_else(|| Some(refresh_token_str.to_string())),
        expires_at: chrono::Utc::now()
            + chrono::Duration::seconds(token_response.expires_in as i64),
    })
}
