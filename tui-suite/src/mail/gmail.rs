use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};

use super::mime::{build_mime_message, encode_for_gmail, EmailMessage};

#[async_trait]
pub trait MailProvider: Send + Sync {
    async fn send(&self, msg: &EmailMessage) -> Result<()>;
    async fn get_user_email(&self) -> Result<String>;
}

pub struct GmailProvider {
    access_token: String,
}

impl GmailProvider {
    pub fn new(access_token: String) -> Self {
        Self { access_token }
    }
}

#[derive(Debug, Serialize)]
struct GmailSendRequest {
    raw: String,
}

#[derive(Debug, Deserialize)]
struct GmailProfile {
    #[serde(rename = "emailAddress")]
    email_address: String,
}

#[async_trait]
impl MailProvider for GmailProvider {
    async fn send(&self, msg: &EmailMessage) -> Result<()> {
        let from = self.get_user_email().await?;
        let mime = build_mime_message(msg, &from);
        let encoded = encode_for_gmail(&mime);

        let client = reqwest::Client::new();
        let response = client
            .post("https://www.googleapis.com/gmail/v1/users/me/messages/send")
            .bearer_auth(&self.access_token)
            .json(&GmailSendRequest { raw: encoded })
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(AppError::Api {
                status: status.as_u16(),
                message: text,
            });
        }

        Ok(())
    }

    async fn get_user_email(&self) -> Result<String> {
        let client = reqwest::Client::new();
        let response = client
            .get("https://www.googleapis.com/gmail/v1/users/me/profile")
            .bearer_auth(&self.access_token)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(AppError::Api {
                status: status.as_u16(),
                message: text,
            });
        }

        let profile: GmailProfile = response.json().await?;
        Ok(profile.email_address)
    }
}
