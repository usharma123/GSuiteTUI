use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use serde::Deserialize;

use crate::error::{AppError, Result};

use super::model::{CalendarEvent, EventStatus, EventTime};

#[derive(Debug)]
pub struct SyncResult {
    pub events: Vec<CalendarEvent>,
    pub sync_token: Option<String>,
}

#[async_trait]
pub trait CalendarProvider: Send + Sync {
    async fn sync(&self, sync_token: Option<String>) -> Result<SyncResult>;
}

pub struct GoogleCalendarProvider {
    access_token: String,
    calendar_id: String,
}

impl GoogleCalendarProvider {
    pub fn new(access_token: String, calendar_id: String) -> Self {
        Self {
            access_token,
            calendar_id,
        }
    }
}

#[derive(Debug, Deserialize)]
struct GoogleEventsResponse {
    items: Option<Vec<GoogleEvent>>,
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
    #[serde(rename = "nextSyncToken")]
    next_sync_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GoogleEvent {
    id: String,
    summary: Option<String>,
    start: Option<GoogleDateTime>,
    end: Option<GoogleDateTime>,
    status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GoogleDateTime {
    #[serde(rename = "dateTime")]
    date_time: Option<String>,
    date: Option<String>,
}

#[async_trait]
impl CalendarProvider for GoogleCalendarProvider {
    async fn sync(&self, sync_token: Option<String>) -> Result<SyncResult> {
        let client = reqwest::Client::new();
        let mut all_events = Vec::new();
        let mut page_token: Option<String> = None;
        let mut final_sync_token: Option<String> = None;

        loop {
            let mut url = format!(
                "https://www.googleapis.com/calendar/v3/calendars/{}/events",
                urlencoding::encode(&self.calendar_id)
            );

            let mut params = vec![];

            if let Some(ref token) = sync_token {
                params.push(format!("syncToken={}", urlencoding::encode(token)));
            } else {
                // Initial sync: get events from 30 days ago to 90 days ahead
                let time_min = Utc::now() - chrono::Duration::days(30);
                let time_max = Utc::now() + chrono::Duration::days(90);
                params.push(format!("timeMin={}", time_min.to_rfc3339()));
                params.push(format!("timeMax={}", time_max.to_rfc3339()));
                params.push("singleEvents=true".to_string());
            }

            if let Some(ref pt) = page_token {
                params.push(format!("pageToken={}", urlencoding::encode(pt)));
            }

            params.push("maxResults=250".to_string());

            if !params.is_empty() {
                url = format!("{}?{}", url, params.join("&"));
            }

            let response = client
                .get(&url)
                .bearer_auth(&self.access_token)
                .send()
                .await?;

            let status = response.status();

            if status == reqwest::StatusCode::GONE {
                // 410 GONE - sync token expired, need full resync
                return Err(AppError::SyncTokenGone);
            }

            if !status.is_success() {
                let text = response.text().await.unwrap_or_default();
                return Err(AppError::Api {
                    status: status.as_u16(),
                    message: text,
                });
            }

            let data: GoogleEventsResponse = response.json().await?;

            if let Some(items) = data.items {
                for item in items {
                    if let Some(event) = convert_google_event(item) {
                        all_events.push(event);
                    }
                }
            }

            if let Some(nst) = data.next_sync_token {
                final_sync_token = Some(nst);
            }

            if let Some(npt) = data.next_page_token {
                page_token = Some(npt);
            } else {
                break;
            }
        }

        Ok(SyncResult {
            events: all_events,
            sync_token: final_sync_token,
        })
    }
}

fn convert_google_event(ge: GoogleEvent) -> Option<CalendarEvent> {
    let summary = ge.summary.unwrap_or_else(|| "(No title)".to_string());

    let start = ge.start.as_ref().and_then(|dt| convert_datetime(dt))?;
    let end = ge.end.as_ref().and_then(|dt| convert_datetime(dt))?;

    let status = match ge.status.as_deref() {
        Some("confirmed") => EventStatus::Confirmed,
        Some("tentative") => EventStatus::Tentative,
        Some("cancelled") => EventStatus::Cancelled,
        _ => EventStatus::Confirmed,
    };

    Some(CalendarEvent {
        id: ge.id,
        summary,
        start,
        end,
        status,
    })
}

fn convert_datetime(gdt: &GoogleDateTime) -> Option<EventTime> {
    if let Some(ref dt_str) = gdt.date_time {
        // Parse ISO 8601 datetime
        DateTime::parse_from_rfc3339(dt_str)
            .ok()
            .map(|dt| EventTime::DateTime(dt.to_utc()))
    } else if let Some(ref date_str) = gdt.date {
        // Parse date only (all-day event)
        NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .ok()
            .map(EventTime::Date)
    } else {
        None
    }
}
