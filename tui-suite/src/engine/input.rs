use crossterm::event::KeyEvent;

use crate::app::keymap::{
    map_calendar_key, map_compose_key, map_editor_key, map_global_key, map_palette_key, Action,
};
use crate::app::{AppState, Scene};
use crate::ui::PaletteCommand;

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

    // Global actions
    let action = map_global_key(key);
    match action {
        Action::Quit => {
            app.should_quit = true;
            return;
        }
        Action::CycleScene => {
            app.scene = app.scene.next();
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

    // Scene-specific actions
    match app.scene {
        Scene::Editor => handle_editor_key(app, key),
        Scene::CalendarWeek => handle_calendar_key(app, key),
        Scene::MailCompose => handle_compose_key(app, key),
        Scene::MailInbox => {}
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
        _ => {}
    }
}

fn handle_compose_key(app: &mut AppState, key: KeyEvent) {
    let action = map_compose_key(key);
    match action {
        Action::SendMail => {
            app.set_status("Sending email...");
            // TODO: Trigger mail send task
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
        PaletteCommand::SwitchToEditor => app.scene = Scene::Editor,
        PaletteCommand::SwitchToCalendar => app.scene = Scene::CalendarWeek,
        PaletteCommand::ComposeEmail => app.open_compose(),
        PaletteCommand::Save => {
            match app.editor.save() {
                Ok(()) => app.set_status("Saved"),
                Err(e) => app.set_status(format!("Save failed: {e}")),
            }
        }
        PaletteCommand::SyncCalendar => {
            app.set_status("Syncing calendar...");
            // TODO: Trigger calendar sync task
        }
        PaletteCommand::Undo => app.editor.undo(),
        PaletteCommand::Redo => app.editor.redo(),
        PaletteCommand::Quit => app.should_quit = true,
        PaletteCommand::LoginGoogle => {
            app.set_status("Starting OAuth flow...");
            // TODO: Trigger OAuth task
        }
    }
}
