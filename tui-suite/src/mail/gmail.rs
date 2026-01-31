use async_trait::async_trait;
use base64::engine::general_purpose::URL_SAFE;
use base64::Engine;
use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};

use super::inbox::{EmailDetail, EmailSummary};
use super::mime::{build_mime_message, encode_for_gmail, EmailMessage};

#[async_trait]
pub trait MailProvider: Send + Sync {
    async fn send(&self, msg: &EmailMessage) -> Result<()>;
    async fn get_user_email(&self) -> Result<String>;
    async fn list_messages(&self, max_results: u32) -> Result<Vec<EmailSummary>>;
    async fn get_message(&self, id: &str) -> Result<EmailDetail>;
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

#[derive(Debug, Deserialize)]
struct GmailMessageList {
    messages: Option<Vec<GmailMessageRef>>,
}

#[derive(Debug, Deserialize)]
struct GmailMessageRef {
    id: String,
    #[serde(rename = "threadId")]
    #[allow(dead_code)]
    thread_id: String, // Keep for future thread view feature
}

#[derive(Debug, Deserialize)]
struct GmailMessage {
    id: String,
    #[serde(rename = "threadId")]
    thread_id: String,
    #[serde(rename = "labelIds")]
    label_ids: Option<Vec<String>>,
    snippet: Option<String>,
    payload: Option<GmailPayload>,
    #[serde(rename = "internalDate")]
    internal_date: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GmailPayload {
    headers: Option<Vec<GmailHeader>>,
    body: Option<GmailBody>,
    parts: Option<Vec<GmailPart>>,
}

#[derive(Debug, Clone, Deserialize)]
struct GmailHeader {
    name: String,
    value: String,
}

#[derive(Debug, Deserialize)]
struct GmailBody {
    data: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GmailPart {
    #[serde(rename = "mimeType")]
    mime_type: Option<String>,
    body: Option<GmailBody>,
    parts: Option<Vec<GmailPart>>,
}

fn get_header(headers: &Option<Vec<GmailHeader>>, name: &str) -> String {
    headers
        .as_ref()
        .and_then(|h| h.iter().find(|header| header.name.eq_ignore_ascii_case(name)))
        .map(|h| h.value.clone())
        .unwrap_or_default()
}

fn decode_body(data: &Option<String>) -> String {
    data.as_ref()
        .and_then(|d| URL_SAFE.decode(d).ok())
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .unwrap_or_default()
}

fn html_to_text(html: &str) -> String {
    html2text::from_read(html.as_bytes(), 80)
}

fn extract_text_body(payload: &Option<GmailPayload>) -> String {
    let Some(payload) = payload else {
        return String::new();
    };

    // Try to find text/plain first
    if let Some(ref parts) = payload.parts {
        for part in parts {
            if part.mime_type.as_deref() == Some("text/plain") {
                if let Some(ref body) = part.body {
                    let text = decode_body(&body.data);
                    if !text.is_empty() {
                        return text;
                    }
                }
            }
            // Recurse into nested parts
            if let Some(ref nested) = part.parts {
                for nested_part in nested {
                    if nested_part.mime_type.as_deref() == Some("text/plain") {
                        if let Some(ref body) = nested_part.body {
                            let text = decode_body(&body.data);
                            if !text.is_empty() {
                                return text;
                            }
                        }
                    }
                }
            }
        }

        // Fallback to text/html and convert
        for part in parts {
            if part.mime_type.as_deref() == Some("text/html") {
                if let Some(ref body) = part.body {
                    let html = decode_body(&body.data);
                    if !html.is_empty() {
                        return html_to_text(&html);
                    }
                }
            }
            // Recurse into nested parts for HTML
            if let Some(ref nested) = part.parts {
                for nested_part in nested {
                    if nested_part.mime_type.as_deref() == Some("text/html") {
                        if let Some(ref body) = nested_part.body {
                            let html = decode_body(&body.data);
                            if !html.is_empty() {
                                return html_to_text(&html);
                            }
                        }
                    }
                }
            }
        }
    }

    // Simple body (non-multipart)
    if let Some(ref body) = payload.body {
        if body.data.is_some() {
            let content = decode_body(&body.data);
            // Check if it looks like HTML
            if content.contains("<html") || content.contains("<body") || content.contains("<div") {
                return html_to_text(&content);
            }
            return content;
        }
    }

    String::new()
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

    async fn list_messages(&self, max_results: u32) -> Result<Vec<EmailSummary>> {
        let client = reqwest::Client::new();

        // Get list of message IDs
        let url = format!(
            "https://www.googleapis.com/gmail/v1/users/me/messages?maxResults={}&labelIds=INBOX",
            max_results
        );

        let response = client
            .get(&url)
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

        let list: GmailMessageList = response.json().await?;
        let message_refs = list.messages.unwrap_or_default();

        // Fetch each message's metadata
        let mut emails = Vec::new();
        for msg_ref in message_refs.iter().take(max_results as usize) {
            let msg_url = format!(
                "https://www.googleapis.com/gmail/v1/users/me/messages/{}?format=metadata&metadataHeaders=From&metadataHeaders=Subject&metadataHeaders=Date",
                msg_ref.id
            );

            let msg_response = client
                .get(&msg_url)
                .bearer_auth(&self.access_token)
                .send()
                .await?;

            if msg_response.status().is_success() {
                let msg: GmailMessage = msg_response.json().await?;

                let from = get_header(&msg.payload.as_ref().and_then(|p| p.headers.clone()), "From");
                let subject = get_header(&msg.payload.as_ref().and_then(|p| p.headers.clone()), "Subject");

                let date = msg
                    .internal_date
                    .as_ref()
                    .and_then(|d| d.parse::<i64>().ok())
                    .and_then(|ms| Utc.timestamp_millis_opt(ms).single())
                    .unwrap_or_else(Utc::now);

                let is_unread = msg
                    .label_ids
                    .as_ref()
                    .map(|labels| labels.contains(&"UNREAD".to_string()))
                    .unwrap_or(false);

                emails.push(EmailSummary {
                    id: msg.id,
                    thread_id: msg.thread_id,
                    from,
                    subject,
                    snippet: msg.snippet.unwrap_or_default(),
                    date,
                    is_unread,
                });
            }
        }

        Ok(emails)
    }

    async fn get_message(&self, id: &str) -> Result<EmailDetail> {
        let client = reqwest::Client::new();
        let url = format!(
            "https://www.googleapis.com/gmail/v1/users/me/messages/{}?format=full",
            id
        );

        let response = client
            .get(&url)
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

        let msg: GmailMessage = response.json().await?;

        let headers = msg.payload.as_ref().and_then(|p| p.headers.clone());
        let from = get_header(&headers, "From");
        let to = get_header(&headers, "To");
        let subject = get_header(&headers, "Subject");

        let date = msg
            .internal_date
            .as_ref()
            .and_then(|d| d.parse::<i64>().ok())
            .and_then(|ms| Utc.timestamp_millis_opt(ms).single())
            .unwrap_or_else(Utc::now);

        let body_text = extract_text_body(&msg.payload);

        Ok(EmailDetail {
            id: msg.id,
            from,
            to,
            subject,
            date,
            body_text,
            body_html: None,
        })
    }
}
