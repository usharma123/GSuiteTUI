use std::io;
use std::io::Write;
use std::time::Duration;

use crossterm::event::{Event, EventStream};
use futures::StreamExt;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use tokio::sync::mpsc;
use tokio::time::interval;

use crate::app::AppState;
use crate::auth::token_store::get_token_store;
use crate::config::Config;
use crate::engine::input::handle_key;
use crate::engine::tasks::TaskResult;
use crate::storage::EventCache;
use crate::ui::render_app;

fn debug_log(msg: &str) {
    use std::fs::OpenOptions;
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/tui-debug.log")
    {
        let timestamp = chrono::Local::now().format("%H:%M:%S%.3f");
        let _ = writeln!(file, "[{}] {}", timestamp, msg);
    }
}

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
            debug_log(&format!("Calendar sync: {} events returned", sync_result.events.len()));

            // Merge incremental sync results with existing events
            // Remove events that are in the sync result (they may be updated or cancelled)
            // Then add non-cancelled events from sync result
            let sync_ids: std::collections::HashSet<_> =
                sync_result.events.iter().map(|e| e.id.clone()).collect();

            // Keep events not in this sync batch
            app.calendar.events.retain(|e| !sync_ids.contains(&e.id));

            // Add non-cancelled events from sync result
            for event in sync_result.events.iter() {
                if event.status != crate::calendar::EventStatus::Cancelled {
                    app.calendar.events.push(event.clone());
                }
            }

            app.calendar.sync_token = sync_result.sync_token.clone();
            debug_log(&format!("Total events after merge: {}", app.calendar.events.len()));
            app.set_status(format!("Synced - {} events total", app.calendar.events.len()));

            // Save to cache
            let mut cache = EventCache::new();
            cache.update(app.calendar.events.clone(), app.calendar.sync_token.clone());
            if let Err(e) = cache.save() {
                debug_log(&format!("Failed to save event cache: {}", e));
            } else {
                debug_log("Event cache saved");
            }
        }
        TaskResult::CalendarSynced(Err(e)) => {
            debug_log(&format!("Calendar sync failed: {}", e));
            app.set_status(format!("Sync failed: {e}"));
        }
        TaskResult::MailSent(Ok(())) => {
            debug_log("Email sent successfully");
            app.set_status("Email sent!");
            app.close_compose();
        }
        TaskResult::MailSent(Err(e)) => {
            debug_log(&format!("Email send failed: {}", e));
            app.set_status(format!("Send failed: {e}"));
        }
        TaskResult::OAuthComplete(Ok(tokens)) => {
            debug_log(&format!("OAuth complete, expires_at: {:?}", tokens.expires_at));
            let store = get_token_store();
            match store.save(&tokens) {
                Ok(()) => {
                    debug_log("Tokens saved successfully");
                    app.set_status("Logged in to Google - tokens saved!");
                }
                Err(e) => {
                    debug_log(&format!("Token save failed: {}", e));
                    app.set_status(format!("Login ok but token save failed: {e}"));
                }
            }
        }
        TaskResult::OAuthComplete(Err(e)) => {
            debug_log(&format!("OAuth failed: {}", e));
            app.set_status(format!("OAuth failed: {e}"));
        }
        TaskResult::TokenRefreshed(Ok(_)) => {
            debug_log("Token refreshed");
            app.set_status("Token refreshed");
        }
        TaskResult::TokenRefreshed(Err(e)) => {
            debug_log(&format!("Token refresh failed: {}", e));
            app.set_status(format!("Token refresh failed: {e}"));
        }
        TaskResult::InboxSynced(Ok(emails)) => {
            debug_log(&format!("Inbox synced: {} emails", emails.len()));
            app.inbox.emails = emails;
            app.set_status(format!("Loaded {} emails", app.inbox.emails.len()));
        }
        TaskResult::InboxSynced(Err(e)) => {
            debug_log(&format!("Inbox sync failed: {}", e));
            app.set_status(format!("Inbox sync failed: {e}"));
        }
        TaskResult::EmailFetched(Ok(detail)) => {
            debug_log(&format!("Email fetched: {}", detail.subject));
            app.inbox.open_email(detail);
        }
        TaskResult::EmailFetched(Err(e)) => {
            debug_log(&format!("Email fetch failed: {}", e));
            app.set_status(format!("Failed to load email: {e}"));
        }
    }
}
