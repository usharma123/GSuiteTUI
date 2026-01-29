use std::io;
use std::time::Duration;

use crossterm::event::{Event, EventStream};
use futures::StreamExt;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use tokio::sync::mpsc;
use tokio::time::interval;

use crate::app::AppState;
use crate::config::Config;
use crate::engine::input::handle_key;
use crate::engine::tasks::TaskResult;
use crate::ui::render_app;

pub async fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> anyhow::Result<()> {
    let (task_tx, mut task_rx) = mpsc::unbounded_channel::<TaskResult>();
    let config = Config::load().unwrap_or_default();
    let mut app = AppState::new(config, task_tx);

    let mut event_stream = EventStream::new();
    let mut render_interval = interval(Duration::from_millis(50));

    loop {
        tokio::select! {
            _ = render_interval.tick() => {
                terminal.draw(|f| render_app(f, &mut app))?;
            }
            Some(result) = task_rx.recv() => {
                handle_task_result(&mut app, result);
            }
            Some(Ok(event)) = event_stream.next() => {
                match event {
                    Event::Key(key) => handle_key(&mut app, key),
                    Event::Resize(_, _) => {}
                    _ => {}
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn handle_task_result(app: &mut AppState, result: TaskResult) {
    match result {
        TaskResult::CalendarSynced(Ok(sync_result)) => {
            app.calendar.events = sync_result.events;
            app.calendar.sync_token = sync_result.sync_token;
            app.set_status(format!("Synced {} events", app.calendar.events.len()));
        }
        TaskResult::CalendarSynced(Err(e)) => {
            app.set_status(format!("Sync failed: {e}"));
        }
        TaskResult::MailSent(Ok(())) => {
            app.set_status("Email sent!");
            app.close_compose();
        }
        TaskResult::MailSent(Err(e)) => {
            app.set_status(format!("Send failed: {e}"));
        }
        TaskResult::OAuthComplete(Ok(tokens)) => {
            app.set_status("Logged in to Google");
            // TODO: Store tokens
            let _ = tokens;
        }
        TaskResult::OAuthComplete(Err(e)) => {
            app.set_status(format!("OAuth failed: {e}"));
        }
        TaskResult::TokenRefreshed(Ok(_)) => {
            app.set_status("Token refreshed");
        }
        TaskResult::TokenRefreshed(Err(e)) => {
            app.set_status(format!("Token refresh failed: {e}"));
        }
    }
}
