use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::error::{AppError, Result};

#[derive(Debug, Clone)]
pub struct SpreadsheetDoc {
    pub id: String,
    pub name: String,
    pub modified_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct SheetTab {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct SpreadsheetMeta {
    pub id: String,
    pub name: String,
    pub sheets: Vec<SheetTab>,
}

#[derive(Debug, Clone)]
pub struct SheetData {
    pub values: Vec<Vec<String>>,
}

pub struct SheetsProvider {
    access_token: String,
    client: reqwest::Client,
}

impl SheetsProvider {
    pub fn new(access_token: String) -> Self {
        Self {
            access_token,
            client: reqwest::Client::new(),
        }
    }

    pub async fn list_spreadsheets(&self, query: &str, max_results: u32) -> Result<Vec<SpreadsheetDoc>> {
        let q = build_spreadsheet_query(query);
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
            .map(|file| SpreadsheetDoc {
                id: file.id,
                name: file.name,
                modified_time: parse_modified_time(file.modified_time.as_deref()),
            })
            .collect();

        Ok(files)
    }

    pub async fn get_spreadsheet_meta(&self, spreadsheet_id: &str) -> Result<SpreadsheetMeta> {
        let url = format!(
            "https://sheets.googleapis.com/v4/spreadsheets/{}?fields=properties.title,sheets(properties(sheetId,title,gridProperties(rowCount,columnCount)))",
            urlencoding::encode(spreadsheet_id),
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

        let meta: SpreadsheetResponse = response.json().await?;
        let sheets = meta
            .sheets
            .unwrap_or_default()
            .into_iter()
            .filter_map(|sheet| sheet.properties)
            .map(|props| SheetTab {
                id: props.sheet_id,
                name: props.title,
            })
            .collect();

        Ok(SpreadsheetMeta {
            id: spreadsheet_id.to_string(),
            name: meta.properties.map(|p| p.title).unwrap_or_default(),
            sheets,
        })
    }

    pub async fn get_values(&self, spreadsheet_id: &str, range_a1: &str) -> Result<SheetData> {
        let url = format!(
            "https://sheets.googleapis.com/v4/spreadsheets/{}/values/{}?valueRenderOption=FORMATTED_VALUE",
            urlencoding::encode(spreadsheet_id),
            urlencoding::encode(range_a1),
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

        let values: ValuesResponse = response.json().await?;
        Ok(SheetData {
            values: values.values.unwrap_or_default(),
        })
    }

    pub async fn update_value(
        &self,
        spreadsheet_id: &str,
        range_a1: &str,
        value: &str,
    ) -> Result<()> {
        let url = format!(
            "https://sheets.googleapis.com/v4/spreadsheets/{}/values/{}?valueInputOption=USER_ENTERED",
            urlencoding::encode(spreadsheet_id),
            urlencoding::encode(range_a1),
        );

        let body = serde_json::json!({
            "range": range_a1,
            "majorDimension": "ROWS",
            "values": [[value]],
        });

        let response = self
            .client
            .put(&url)
            .bearer_auth(&self.access_token)
            .json(&body)
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

fn build_spreadsheet_query(query: &str) -> String {
    let base = "mimeType='application/vnd.google-apps.spreadsheet' and trashed=false";
    let trimmed = query.trim();
    if trimmed.is_empty() {
        base.to_string()
    } else {
        let escaped = trimmed.replace('\'', "\\'");
        format!("{base} and name contains '{escaped}'")
    }
}

#[derive(Debug, Deserialize)]
struct SpreadsheetResponse {
    properties: Option<SpreadsheetProperties>,
    sheets: Option<Vec<SheetPropertiesWrapper>>,
}

#[derive(Debug, Deserialize)]
struct SpreadsheetProperties {
    title: String,
}

#[derive(Debug, Deserialize)]
struct SheetPropertiesWrapper {
    properties: Option<SheetProperties>,
}

#[derive(Debug, Deserialize)]
struct SheetProperties {
    #[serde(rename = "sheetId")]
    sheet_id: i64,
    title: String,
}

#[derive(Debug, Deserialize)]
struct ValuesResponse {
    values: Option<Vec<Vec<String>>>,
}
