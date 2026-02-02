use crossterm::event::{KeyCode, KeyEvent};

use crate::app::keymap::{
    map_calendar_key, map_compose_key, map_drive_key, map_editor_key, map_global_key,
    map_inbox_key, map_palette_key, Action,
};
use crate::app::{AppState, Scene};
use crate::auth::{token_store::get_token_store, OAuthFlow};
use crate::calendar::sync::GoogleCalendarProvider;
use crate::drive::DriveProvider;
use crate::editor::markdown::markdown_to_html;
use crate::engine::tasks::{
    spawn_calendar_sync, spawn_drive_create, spawn_drive_list, spawn_drive_open, spawn_drive_save,
    spawn_email_fetch, spawn_inbox_sync, spawn_mail_send, spawn_oauth, CalendarSyncResult,
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
                if palette.is_drive_search() {
                    if let Some(doc) = palette.selected_drive_doc() {
                        open_drive_doc(app, doc);
                    }
                    app.close_palette();
                } else if let Some(cmd) = palette.selected_command() {
                    let should_close = execute_palette_command(app, cmd);
                    if should_close {
                        app.close_palette();
                    }
                }
            }
            Action::ClosePalette => app.close_palette(),
            Action::PaletteType(c) => {
                palette.type_char(c);
                if palette.is_drive_search() {
                    trigger_drive_search(app);
                }
            }
            Action::PaletteBackspace => {
                palette.backspace();
                if palette.is_drive_search() {
                    trigger_drive_search(app);
                }
            }
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
                if app.scene == Scene::DriveBrowser {
                    open_drive_browser(app);
                }
                return;
            }
            Action::OpenPalette => {
                app.open_palette();
                return;
            }
            Action::Save => {
                handle_editor_save(app);
                return;
            }
            Action::SyncCalendar => {
                trigger_calendar_sync(app);
                return;
            }
            Action::OpenDriveBrowser => {
                open_drive_browser(app);
                return;
            }
            Action::CreateDriveDoc => {
                start_create_drive_doc(app);
                return;
            }
            Action::ClosePalette => {
                if app.palette.is_some() {
                    app.close_palette();
                    return;
                }
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
        Scene::DriveBrowser => handle_drive_key(app, key),
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
        Action::SyncCalendar => trigger_calendar_sync(app),
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

fn handle_drive_key(app: &mut AppState, key: KeyEvent) {
    let action = map_drive_key(key);
    match action {
        Action::DriveUp => app.drive.move_up(),
        Action::DriveDown => app.drive.move_down(),
        Action::DriveSelect => {
            if app.drive.is_creating() {
                submit_create_drive_doc(app);
            } else if let Some(doc) = app.drive.selected_doc() {
                open_drive_doc(app, doc);
            }
        }
        Action::DriveType(c) => {
            app.drive.type_char(c);
            if !app.drive.is_creating() {
                trigger_drive_browser_list(app);
            }
        }
        Action::DriveBackspace => {
            app.drive.backspace();
            if !app.drive.is_creating() {
                trigger_drive_browser_list(app);
            }
        }
        Action::DriveCancel => {
            if app.drive.is_creating() {
                app.drive.cancel_create();
            } else if !app.drive.query.is_empty() {
                app.drive.clear_query();
                trigger_drive_browser_list(app);
            }
        }
        _ => {}
    }
}

fn execute_palette_command(app: &mut AppState, cmd: PaletteCommand) -> bool {
    match cmd {
        PaletteCommand::SwitchToEditor => {
            app.scene = Scene::Editor;
            app.update_status_hint();
            true
        }
        PaletteCommand::SwitchToCalendar => {
            app.scene = Scene::CalendarWeek;
            app.update_status_hint();
            true
        }
        PaletteCommand::OpenDriveDoc => {
            if let Some(ref mut palette) = app.palette {
                palette.enter_drive_search();
            }
            trigger_drive_search(app);
            false
        }
        PaletteCommand::ComposeEmail => {
            app.open_compose();
            app.update_status_hint();
            true
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
            true
        }
        PaletteCommand::Save => {
            handle_editor_save(app);
            true
        }
        PaletteCommand::SyncCalendar => {
            trigger_calendar_sync(app);
            true
        }
        PaletteCommand::Undo => {
            app.editor.undo();
            true
        }
        PaletteCommand::Redo => {
            app.editor.redo();
            true
        }
        PaletteCommand::Quit => {
            app.should_quit = true;
            true
        }
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
                                return true;
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
            true
        }
    }
}

fn handle_editor_save(app: &mut AppState) {
    if let Some(doc) = app.editor.drive_doc() {
        let store = get_token_store();
        match store.load() {
            Ok(Some(tokens)) if !tokens.is_expired() => {
                app.set_status("Saving to Drive...");
                let access_token = tokens.access_token.clone();
                let body_html = markdown_to_html(&app.editor.markdown());
                let html = format!(
                    "<!doctype html><html><body>{}</body></html>",
                    body_html
                );
                let tx = app.task_tx.clone();
                spawn_drive_save(tx, move || async move {
                    let provider = DriveProvider::new(access_token);
                    provider.update_doc_html(&doc.id, &html).await
                });
            }
            Ok(Some(_)) => app.set_status("Token expired - please Login Google again"),
            Ok(None) => app.set_status("Not logged in - use Login Google first"),
            Err(e) => app.set_status(format!("Token error: {e}")),
        }
    } else {
        match app.editor.save_local() {
            Ok(()) => app.set_status("Saved"),
            Err(e) => app.set_status(format!("Save failed: {e}")),
        }
    }
}

fn trigger_drive_search(app: &mut AppState) {
    let query = match app.palette.as_ref() {
        Some(palette) => palette.query.clone(),
        None => return,
    };

    let store = get_token_store();
    match store.load() {
        Ok(Some(tokens)) if !tokens.is_expired() => {
            if let Some(ref mut palette) = app.palette {
                palette.set_drive_loading();
            }
            let access_token = tokens.access_token.clone();
            let tx = app.task_tx.clone();
            spawn_drive_list(tx, move || async move {
                let provider = DriveProvider::new(access_token);
                provider.list_docs(&query, 20).await
            });
        }
        Ok(Some(_)) => {
            if let Some(ref mut palette) = app.palette {
                palette.set_drive_error("Token expired - please Login Google again".to_string());
            }
        }
        Ok(None) => {
            if let Some(ref mut palette) = app.palette {
                palette.set_drive_error("Not logged in - use Login Google first".to_string());
            }
        }
        Err(e) => {
            if let Some(ref mut palette) = app.palette {
                palette.set_drive_error(format!("Token error: {e}"));
            }
        }
    }
}

fn open_drive_browser(app: &mut AppState) {
    app.scene = Scene::DriveBrowser;
    app.update_status_hint();
    app.drive.reset();
    trigger_drive_browser_list(app);
}

fn trigger_drive_browser_list(app: &mut AppState) {
    let query = app.drive.query.clone();
    let store = get_token_store();
    match store.load() {
        Ok(Some(tokens)) if !tokens.is_expired() => {
            app.drive.set_loading();
            let access_token = tokens.access_token.clone();
            let tx = app.task_tx.clone();
            spawn_drive_list(tx, move || async move {
                let provider = DriveProvider::new(access_token);
                provider.list_docs(&query, 50).await
            });
        }
        Ok(Some(_)) => {
            app.drive
                .set_error("Token expired - please Login Google again".to_string());
        }
        Ok(None) => {
            app.drive
                .set_error("Not logged in - use Login Google first".to_string());
        }
        Err(e) => {
            app.drive.set_error(format!("Token error: {e}"));
        }
    }
}

fn start_create_drive_doc(app: &mut AppState) {
    let store = get_token_store();
    match store.load() {
        Ok(Some(tokens)) if !tokens.is_expired() => {
            if app.scene != Scene::DriveBrowser {
                open_drive_browser(app);
            }
            app.drive.start_create();
            app.set_status("Enter new Drive doc name");
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

fn submit_create_drive_doc(app: &mut AppState) {
    let name = app.drive.new_doc_name.trim().to_string();
    if name.is_empty() {
        app.set_status("Document name required");
        return;
    }

    let store = get_token_store();
    match store.load() {
        Ok(Some(tokens)) if !tokens.is_expired() => {
            app.set_status("Creating Drive doc...");
            app.drive.cancel_create();
            let access_token = tokens.access_token.clone();
            let tx = app.task_tx.clone();
            spawn_drive_create(tx, move || async move {
                let provider = DriveProvider::new(access_token);
                provider.create_doc(&name).await
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

fn trigger_calendar_sync(app: &mut AppState) {
    debug_log("SyncCalendar triggered");

    // Check for stored tokens
    let store = get_token_store();
    debug_log("Got token store, attempting load...");
    match store.load() {
        Ok(Some(tokens)) if !tokens.is_expired() => {
            debug_log(&format!(
                "Tokens found, expires_at: {:?}, syncing...",
                tokens.expires_at
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

fn open_drive_doc(app: &mut AppState, doc: crate::drive::DriveDoc) {
    let store = get_token_store();
    match store.load() {
        Ok(Some(tokens)) if !tokens.is_expired() => {
            app.set_status("Loading Drive doc...");
            let access_token = tokens.access_token.clone();
            let tx = app.task_tx.clone();
            spawn_drive_open(tx, move || async move {
                let provider = DriveProvider::new(access_token);
                provider.export_doc_markdown(&doc).await
            });
        }
        Ok(Some(_)) => app.set_status("Token expired - please Login Google again"),
        Ok(None) => app.set_status("Not logged in - use Login Google first"),
        Err(e) => app.set_status(format!("Token error: {e}")),
    }
}
