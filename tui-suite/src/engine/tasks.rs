use tokio::sync::mpsc;

use crate::auth::TokenPair;
use crate::calendar::CalendarEvent;
use crate::error::Result;
use crate::drive::{DriveDoc, DriveDocContent};
use crate::mail::{EmailDetail, EmailSummary};
use crate::sheets::{SheetData, SpreadsheetDoc, SpreadsheetMeta};

#[derive(Debug)]
pub enum TaskResult {
    CalendarSynced(Result<CalendarSyncResult>),
    MailSent(Result<()>),
    InboxSynced(Result<Vec<EmailSummary>>),
    EmailFetched(Result<EmailDetail>),
    OAuthComplete(Result<TokenPair>),
    TokenRefreshed(Result<TokenPair>),
    DriveDocsListed(Result<Vec<DriveDoc>>),
    DriveDocOpened(Result<DriveDocContent>),
    DriveDocSaved(Result<()>),
    DriveDocCreated(Result<DriveDoc>),
    SheetsListed(Result<Vec<SpreadsheetDoc>>),
    SheetsOpened(Result<SpreadsheetMeta>),
    SheetsFetched(Result<SheetData>),
    SheetsUpdated(Result<()>),
}

#[derive(Debug)]
pub struct CalendarSyncResult {
    pub events: Vec<CalendarEvent>,
    pub sync_token: Option<String>,
}

pub fn spawn_calendar_sync<F, Fut>(tx: mpsc::UnboundedSender<TaskResult>, sync_fn: F)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<CalendarSyncResult>> + Send,
{
    tokio::spawn(async move {
        let result = sync_fn().await;
        let _ = tx.send(TaskResult::CalendarSynced(result));
    });
}

pub fn spawn_mail_send<F, Fut>(tx: mpsc::UnboundedSender<TaskResult>, send_fn: F)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<()>> + Send,
{
    tokio::spawn(async move {
        let result = send_fn().await;
        let _ = tx.send(TaskResult::MailSent(result));
    });
}

pub fn spawn_oauth<F, Fut>(tx: mpsc::UnboundedSender<TaskResult>, oauth_fn: F)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<TokenPair>> + Send,
{
    tokio::spawn(async move {
        let result = oauth_fn().await;
        let _ = tx.send(TaskResult::OAuthComplete(result));
    });
}

pub fn spawn_inbox_sync<F, Fut>(tx: mpsc::UnboundedSender<TaskResult>, sync_fn: F)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<Vec<EmailSummary>>> + Send,
{
    tokio::spawn(async move {
        let result = sync_fn().await;
        let _ = tx.send(TaskResult::InboxSynced(result));
    });
}

pub fn spawn_email_fetch<F, Fut>(tx: mpsc::UnboundedSender<TaskResult>, fetch_fn: F)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<EmailDetail>> + Send,
{
    tokio::spawn(async move {
        let result = fetch_fn().await;
        let _ = tx.send(TaskResult::EmailFetched(result));
    });
}

pub fn spawn_drive_list<F, Fut>(tx: mpsc::UnboundedSender<TaskResult>, list_fn: F)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<Vec<DriveDoc>>> + Send,
{
    tokio::spawn(async move {
        let result = list_fn().await;
        let _ = tx.send(TaskResult::DriveDocsListed(result));
    });
}

pub fn spawn_drive_open<F, Fut>(tx: mpsc::UnboundedSender<TaskResult>, open_fn: F)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<DriveDocContent>> + Send,
{
    tokio::spawn(async move {
        let result = open_fn().await;
        let _ = tx.send(TaskResult::DriveDocOpened(result));
    });
}

pub fn spawn_drive_save<F, Fut>(tx: mpsc::UnboundedSender<TaskResult>, save_fn: F)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<()>> + Send,
{
    tokio::spawn(async move {
        let result = save_fn().await;
        let _ = tx.send(TaskResult::DriveDocSaved(result));
    });
}

pub fn spawn_drive_create<F, Fut>(tx: mpsc::UnboundedSender<TaskResult>, create_fn: F)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<DriveDoc>> + Send,
{
    tokio::spawn(async move {
        let result = create_fn().await;
        let _ = tx.send(TaskResult::DriveDocCreated(result));
    });
}

pub fn spawn_sheets_list<F, Fut>(tx: mpsc::UnboundedSender<TaskResult>, list_fn: F)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<Vec<SpreadsheetDoc>>> + Send,
{
    tokio::spawn(async move {
        let result = list_fn().await;
        let _ = tx.send(TaskResult::SheetsListed(result));
    });
}

pub fn spawn_sheets_open<F, Fut>(tx: mpsc::UnboundedSender<TaskResult>, open_fn: F)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<SpreadsheetMeta>> + Send,
{
    tokio::spawn(async move {
        let result = open_fn().await;
        let _ = tx.send(TaskResult::SheetsOpened(result));
    });
}

pub fn spawn_sheets_fetch<F, Fut>(tx: mpsc::UnboundedSender<TaskResult>, fetch_fn: F)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<SheetData>> + Send,
{
    tokio::spawn(async move {
        let result = fetch_fn().await;
        let _ = tx.send(TaskResult::SheetsFetched(result));
    });
}

pub fn spawn_sheets_update<F, Fut>(tx: mpsc::UnboundedSender<TaskResult>, update_fn: F)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<()>> + Send,
{
    tokio::spawn(async move {
        let result = update_fn().await;
        let _ = tx.send(TaskResult::SheetsUpdated(result));
    });
}
