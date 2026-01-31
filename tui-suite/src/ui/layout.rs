use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::Frame;

use crate::app::{AppState, Scene};

pub fn render_app(f: &mut Frame, app: &mut AppState) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(f.area());

    // Main content area
    match app.scene {
        Scene::Editor => app.editor.render(f, root[0]),
        Scene::CalendarWeek => app.calendar.render_week(f, root[0]),
        Scene::MailCompose => {
            if let Some(ref mut compose) = app.compose {
                compose.render(f, root[0]);
            }
        }
        Scene::MailInbox => {
            app.inbox.render(f, root[0]);
        }
    }

    // Status line
    app.status.render(f, root[1]);

    // Palette overlay (if open)
    if let Some(ref mut palette) = app.palette {
        palette.render(f, f.area());
    }
}
