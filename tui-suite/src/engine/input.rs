use crossterm::event::{KeyCode, KeyEvent};

use crate::app::keymap::{
    map_calendar_key, map_compose_key, map_editor_key, map_global_key, map_inbox_key,
    map_palette_key, Action,
};
use crate::app::{AppState, Scene};
use crate::auth::{token_store::get_token_store, OAuthFlow};
use crate::calendar::sync::GoogleCalendarProvider;
use crate::engine::tasks::{
    spawn_calendar_sync, spawn_email_fetch, spawn_inbox_sync, spawn_mail_send, spawn_oauth,
    CalendarSyncResult,
};
use crate::mail::gmail::{GmailProvider, MailProvider};
use crate::mail::mime::EmailMessage;
use crate::ui::PaletteCommand;
use std::io::Write;

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

pub fn handle_key(app: &mut AppState, key: KeyEvent) {
    // If palette is open, handle palette input first
    if let Some(ref mut palette) = app.palette {
        let action = map_palette_key(key);
        match action {
            Action::PaletteUp => palette.move_up(),
            Action::PaletteDown => palette.move_down(),
            Action::PaletteSelect => {
                if let Some(cmd) = palette.selected_command() {
                    execute_palette_command(app, cmd);
                }
                app.close_palette();
            }
            Action::ClosePalette => app.close_palette(),
            Action::PaletteType(c) => palette.type_char(c),
            Action::PaletteBackspace => palette.backspace(),
            _ => {}
        }
        return;
    }

    // Global actions - but skip Tab when in MailCompose scene
    let is_tab = matches!(key.code, KeyCode::Tab | KeyCode::BackTab);
    if !(is_tab && app.scene == Scene::MailCompose) {
        let action = map_global_key(key);
        match action {
            Action::Quit => {
                app.should_quit = true;
                return;
            }
            Action::CycleScene => {
                app.scene = app.scene.next();
                app.update_status_hint();
                return;
            }
            Action::OpenPalette => {
                app.open_palette();
                return;
            }
            Action::Save => {
                match app.editor.save() {
                    Ok(()) => app.set_status("Saved"),
                    Err(e) => app.set_status(format!("Save failed: {e}")),
                }
                return;
            }
            Action::ClosePalette => {
                app.close_palette();
                return;
            }
            _ => {}
        }
    }

    // Scene-specific actions
    match app.scene {
        Scene::Editor => handle_editor_key(app, key),
        Scene::CalendarWeek => handle_calendar_key(app, key),
        Scene::MailCompose => handle_compose_key(app, key),
        Scene::MailInbox => handle_inbox_key(app, key),
    }
}

fn handle_editor_key(app: &mut AppState, key: KeyEvent) {
    let action = map_editor_key(key);
    match action {
        Action::MoveUp => app.editor.move_up(),
        Action::MoveDown => app.editor.move_down(),
        Action::MoveLeft => app.editor.move_left(),
        Action::MoveRight => app.editor.move_right(),
        Action::Home => app.editor.move_home(),
        Action::End => app.editor.move_end(),
        Action::InsertChar(c) => app.editor.insert_char(c),
        Action::Newline => app.editor.insert_newline(),
        Action::Backspace => app.editor.backspace(),
        Action::Delete => app.editor.delete(),
        Action::Undo => app.editor.undo(),
        Action::Redo => app.editor.redo(),
        Action::ToggleBold => app.editor.toggle_bold(),
        Action::ToggleItalic => app.editor.toggle_italic(),
        Action::InsertHeading(level) => app.editor.insert_heading(level),
        _ => {}
    }
}

fn handle_calendar_key(app: &mut AppState, key: KeyEvent) {
    let action = map_calendar_key(key);
    match action {
        Action::PrevWeek => app.calendar.prev_week(),
        Action::NextWeek => app.calendar.next_week(),
        Action::ScrollUp => app.calendar.scroll_up(),
        Action::ScrollDown => app.calendar.scroll_down(),
        Action::JumpToNow => app.calendar.jump_to_now(),
        Action::SelectPrevDay => app.calendar.select_prev_day(),
        Action::SelectNextDay => app.calendar.select_next_day(),
        _ => {}
    }
}

fn handle_inbox_key(app: &mut AppState, key: KeyEvent) {
    use crate::mail::InboxViewMode;

    let action = map_inbox_key(key);
    match action {
        Action::MoveDown => {
            if app.inbox.view_mode == InboxViewMode::Reading {
                app.inbox.scroll_down();
            } else {
                app.inbox.select_next();
            }
        }
        Action::MoveUp => {
            if app.inbox.view_mode == InboxViewMode::Reading {
                app.inbox.scroll_up();
            } else {
                app.inbox.select_prev();
            }
        }
        Action::OpenEmail => {
            if app.inbox.view_mode == InboxViewMode::List {
                if let Some(email) = app.inbox.selected_email() {
                    let email_id = email.id.clone();
                    let store = get_token_store();
                    if let Ok(Some(tokens)) = store.load() {
                        if !tokens.is_expired() {
                            app.set_status("Loading email...");
                            let access_token = tokens.access_token.clone();
                            let tx = app.task_tx.clone();
                            spawn_email_fetch(tx, move || async move {
                                let provider = GmailProvider::new(access_token);
                                use crate::mail::gmail::MailProvider;
                                provider.get_message(&email_id).await
                            });
                        }
                    }
                }
            }
        }
        Action::CloseEmail => {
            if app.inbox.view_mode == InboxViewMode::Reading {
                app.inbox.close_email();
            } else {
                app.scene = Scene::Editor;
            }
        }
        Action::RefreshInbox => {
            let store = get_token_store();
            match store.load() {
                Ok(Some(tokens)) if !tokens.is_expired() => {
                    app.set_status("Refreshing inbox...");
                    let access_token = tokens.access_token.clone();
                    let tx = app.task_tx.clone();
                    spawn_inbox_sync(tx, move || async move {
                        let provider = GmailProvider::new(access_token);
                        use crate::mail::gmail::MailProvider;
                        provider.list_messages(20).await
                    });
                }
                Ok(Some(_)) => {
                    app.set_status("Token expired - please Login Google again");
                }
                Ok(None) => {
                    app.set_status("Not logged in - use Login Google first");
                }
                Err(e) => {
                    app.set_status(format!("Token error: {e}"));
                }
            }
        }
        _ => {}
    }
}

fn handle_compose_key(app: &mut AppState, key: KeyEvent) {
    let action = map_compose_key(key);
    match action {
        Action::SendMail => {
            // Extract compose data first to avoid borrowing issues
            let (to, subject, body_markdown) = match &app.compose {
                Some(c) => {
                    if c.to.trim().is_empty() {
                        app.set_status("Cannot send: 'To' field is empty");
                        return;
                    }
                    if c.subject.trim().is_empty() {
                        app.set_status("Cannot send: 'Subject' field is empty");
                        return;
                    }
                    (c.to.clone(), c.subject.clone(), c.body.to_string())
                }
                None => {
                    app.set_status("No email to send");
                    return;
                }
            };

            // Check for tokens
            let store = get_token_store();
            match store.load() {
                Ok(Some(tokens)) if !tokens.is_expired() => {
                    app.set_status("Sending email...");
                    let access_token = tokens.access_token.clone();
                    let email_msg = EmailMessage {
                        to,
                        subject,
                        body_markdown,
                    };
                    let tx = app.task_tx.clone();
                    spawn_mail_send(tx, move || async move {
                        let provider = GmailProvider::new(access_token);
                        provider.send(&email_msg).await
                    });
                }
                Ok(Some(_)) => {
                    app.set_status("Token expired - please Login Google again");
                }
                Ok(None) => {
                    app.set_status("Not logged in - use Login Google first");
                }
                Err(e) => {
                    app.set_status(format!("Token load error: {e}"));
                }
            }
        }
        Action::CycleField => {
            if let Some(ref mut compose) = app.compose {
                compose.cycle_focus();
            }
        }
        _ => {
            // Forward to compose editor
            if let Some(ref mut compose) = app.compose {
                match action {
                    Action::InsertChar(c) => compose.insert_char(c),
                    Action::Newline => compose.insert_newline(),
                    Action::Backspace => compose.backspace(),
                    Action::MoveUp => compose.move_up(),
                    Action::MoveDown => compose.move_down(),
                    Action::MoveLeft => compose.move_left(),
                    Action::MoveRight => compose.move_right(),
                    _ => {}
                }
            }
        }
    }
}

fn execute_palette_command(app: &mut AppState, cmd: PaletteCommand) {
    match cmd {
        PaletteCommand::SwitchToEditor => {
            app.scene = Scene::Editor;
            app.update_status_hint();
        }
        PaletteCommand::SwitchToCalendar => {
            app.scene = Scene::CalendarWeek;
            app.update_status_hint();
        }
        PaletteCommand::ComposeEmail => {
            app.open_compose();
            app.update_status_hint();
        }
        PaletteCommand::OpenInbox => {
            app.open_inbox();
            app.update_status_hint();
            // Auto-refresh on open
            let store = get_token_store();
            if let Ok(Some(tokens)) = store.load() {
                if !tokens.is_expired() {
                    app.set_status("Loading inbox...");
                    let access_token = tokens.access_token.clone();
                    let tx = app.task_tx.clone();
                    spawn_inbox_sync(tx, move || async move {
                        let provider = GmailProvider::new(access_token);
                        use crate::mail::gmail::MailProvider;
                        provider.list_messages(20).await
                    });
                }
            }
        }
        PaletteCommand::Save => {
            match app.editor.save() {
                Ok(()) => app.set_status("Saved"),
                Err(e) => app.set_status(format!("Save failed: {e}")),
            }
        }
        PaletteCommand::SyncCalendar => {
            debug_log("SyncCalendar triggered");

            // Check for stored tokens
            let store = get_token_store();
            debug_log("Got token store, attempting load...");
            match store.load() {
                Ok(Some(tokens)) if !tokens.is_expired() => {
                    debug_log(&format!(
                        "Tokens found, expires_at: {:?}, syncing...", tokens.expires_at
                    ));
                    app.set_status("Syncing calendar...");
                    let access_token = tokens.access_token.clone();
                    let calendar_id = app.config.calendar_id.clone();
                    let sync_token = app.calendar.sync_token.clone();
                    let tx = app.task_tx.clone();
                    spawn_calendar_sync(tx, move || async move {
                        let provider = GoogleCalendarProvider::new(access_token, calendar_id);
                        use crate::calendar::sync::CalendarProvider;
                        let result = provider.sync(sync_token).await;
                        let r = result?;
                        Ok(CalendarSyncResult {
                            events: r.events,
                            sync_token: r.sync_token,
                        })
                    });
                }
                Ok(Some(_)) => {
                    debug_log("Token expired");
                    app.set_status("Token expired - please Login Google again");
                }
                Ok(None) => {
                    debug_log("No tokens found in store");
                    app.set_status("Not logged in - use Login Google first");
                }
                Err(e) => {
                    debug_log(&format!("Token load error: {}", e));
                    app.set_status(format!("Token load error: {e}"));
                }
            }
        }
        PaletteCommand::Undo => app.editor.undo(),
        PaletteCommand::Redo => app.editor.redo(),
        PaletteCommand::Quit => app.should_quit = true,
        PaletteCommand::LoginGoogle => {
            debug_log(&format!(
                "LoginGoogle triggered, client_id: {:?}, client_secret: {:?}",
                app.config.google_client_id,
                app.config.google_client_secret.as_ref().map(|_| "[REDACTED]")
            ));

            // Check if we have client credentials configured
            let client_id = app.config.google_client_id.clone();
            let client_secret = app.config.google_client_secret.clone();

            match (client_id, client_secret) {
                (Some(id), Some(secret)) => {
                    debug_log(&format!("Creating OAuthFlow with id: {}", id));
                    match OAuthFlow::new(&id, &secret) {
                        Ok(flow) => {
                            app.set_status("Opening browser for Google login...");
                            debug_log(&format!("OAuthFlow created, auth_url: {}", flow.auth_url()));
                            if let Err(e) = flow.open_browser() {
                                debug_log(&format!("Failed to open browser: {}", e));
                                app.set_status(format!("Failed to open browser: {e}"));
                                return;
                            }
                            debug_log("Browser opened, spawning oauth task");
                            let tx = app.task_tx.clone();
                            spawn_oauth(tx, move || async move { flow.wait_for_callback().await });
                        }
                        Err(e) => {
                            debug_log(&format!("OAuth setup failed: {}", e));
                            app.set_status(format!("OAuth setup failed: {e}"));
                        }
                    }
                }
                _ => {
                    debug_log("Missing credentials");
                    app.set_status("Missing credentials - see ~/.config/term-workspace/config.json");
                }
            }
        }
    }
}
