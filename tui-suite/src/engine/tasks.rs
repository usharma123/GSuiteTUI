use tokio::sync::mpsc;

use crate::auth::TokenPair;
use crate::calendar::CalendarEvent;
use crate::error::Result;
use crate::mail::{EmailDetail, EmailSummary};

#[derive(Debug)]
pub enum TaskResult {
    CalendarSynced(Result<CalendarSyncResult>),
    MailSent(Result<()>),
    InboxSynced(Result<Vec<EmailSummary>>),
    EmailFetched(Result<EmailDetail>),
    OAuthComplete(Result<TokenPair>),
    TokenRefreshed(Result<TokenPair>),
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
