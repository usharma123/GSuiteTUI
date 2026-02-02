use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::error::{AppError, Result};

#[derive(Debug, Clone)]
pub struct DriveDoc {
    pub id: String,
    pub name: String,
    pub modified_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct DriveDocContent {
    pub doc: DriveDoc,
    pub markdown: String,
}

pub struct DriveProvider {
    access_token: String,
    client: reqwest::Client,
}

impl DriveProvider {
    pub fn new(access_token: String) -> Self {
        Self {
            access_token,
            client: reqwest::Client::new(),
        }
    }

    pub async fn list_docs(&self, query: &str, max_results: u32) -> Result<Vec<DriveDoc>> {
        let q = build_drive_query(query);
        let url = format!(
            "https://www.googleapis.com/drive/v3/files?q={}&pageSize={}&orderBy=modifiedTime desc&fields=files(id,name,modifiedTime,mimeType)",
            urlencoding::encode(&q),
            max_results.max(1),
        );

        let response = self
            .client
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

        let list: DriveFileList = response.json().await?;
        let files = list
            .files
            .unwrap_or_default()
            .into_iter()
            .map(|file| DriveDoc {
                id: file.id,
                name: file.name,
                modified_time: parse_modified_time(file.modified_time.as_deref()),
            })
            .collect();
        Ok(files)
    }

    pub async fn export_doc_html(&self, file_id: &str) -> Result<String> {
        let url = format!(
            "https://www.googleapis.com/drive/v3/files/{}/export?mimeType={}",
            file_id,
            urlencoding::encode("text/html")
        );

        let response = self
            .client
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

        let html = response.text().await?;
        Ok(html)
    }

    pub async fn export_doc_markdown(&self, doc: &DriveDoc) -> Result<DriveDocContent> {
        let html = self.export_doc_html(&doc.id).await?;
        let cleaned = clean_google_docs_html(&html);
        let mut markdown = html2md::parse_html_extended(&cleaned);
        if markdown.contains("<span") || markdown.contains("style=") {
            markdown = html2text::from_read(cleaned.as_bytes(), 100);
        }
        Ok(DriveDocContent {
            doc: doc.clone(),
            markdown,
        })
    }

    pub async fn update_doc_html(&self, file_id: &str, html: &str) -> Result<()> {
        let boundary = format!("==============={}", uuid::Uuid::new_v4());
        let metadata = serde_json::json!({
            "mimeType": "application/vnd.google-apps.document",
        });
        let body = build_multipart_related(
            &boundary,
            metadata.to_string(),
            "text/html",
            html.as_bytes(),
        );

        let url = format!(
            "https://www.googleapis.com/upload/drive/v3/files/{}?uploadType=multipart",
            file_id
        );

        let response = self
            .client
            .patch(&url)
            .bearer_auth(&self.access_token)
            .header(
                reqwest::header::CONTENT_TYPE,
                format!("multipart/related; boundary={}", boundary),
            )
            .body(body)
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
}

#[derive(Debug, Deserialize)]
struct DriveFileList {
    files: Option<Vec<DriveFile>>,
}

#[derive(Debug, Deserialize)]
struct DriveFile {
    id: String,
    name: String,
    #[serde(rename = "modifiedTime")]
    modified_time: Option<String>,
    #[allow(dead_code)]
    #[serde(rename = "mimeType")]
    mime_type: Option<String>,
}

fn parse_modified_time(value: Option<&str>) -> Option<DateTime<Utc>> {
    value
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
}

fn build_drive_query(query: &str) -> String {
    let base = "mimeType='application/vnd.google-apps.document' and trashed=false";
    let trimmed = query.trim();
    if trimmed.is_empty() {
        base.to_string()
    } else {
        let escaped = escape_drive_query_value(trimmed);
        format!("{base} and name contains '{escaped}'")
    }
}

fn escape_drive_query_value(value: &str) -> String {
    value.replace('\'', "\\'")
}

fn build_multipart_related(
    boundary: &str,
    metadata_json: String,
    mime_type: &str,
    content: &[u8],
) -> Vec<u8> {
    let mut body = Vec::new();
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(b"Content-Type: application/json; charset=UTF-8\r\n\r\n");
    body.extend_from_slice(metadata_json.as_bytes());
    body.extend_from_slice(b"\r\n");
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(format!("Content-Type: {}\r\n\r\n", mime_type).as_bytes());
    body.extend_from_slice(content);
    body.extend_from_slice(b"\r\n");
    body.extend_from_slice(format!("--{}--", boundary).as_bytes());
    body
}

fn clean_google_docs_html(input: &str) -> String {
    let mut html = input.to_string();

    html = strip_block_tag(html, "style");
    html = strip_block_tag(html, "head");

    html = strip_opening_tag(html, "span");
    html = html.replace("</span>", "");
    html = strip_opening_tag(html, "meta");
    html = strip_opening_tag(html, "link");

    html
}

fn strip_block_tag(mut html: String, tag: &str) -> String {
    let start_tag = format!("<{}", tag);
    let end_tag = format!("</{}>", tag);
    loop {
        let Some(start) = html.find(&start_tag) else { break };
        let Some(end_rel) = html[start..].find(&end_tag) else {
            break;
        };
        let end = start + end_rel + end_tag.len();
        html.replace_range(start..end, "");
    }
    html
}

fn strip_opening_tag(mut html: String, tag: &str) -> String {
    let start_tag = format!("<{}", tag);
    loop {
        let Some(start) = html.find(&start_tag) else { break };
        let Some(close_rel) = html[start..].find('>') else {
            break;
        };
        let end = start + close_rel + 1;
        html.replace_range(start..end, "");
    }
    html
}
