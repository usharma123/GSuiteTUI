use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::{AppState, Scene};

pub fn render_app(f: &mut Frame, app: &mut AppState) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(f.area());

    render_tabs(f, root[0], app.scene);

    // Main content area
    match app.scene {
        Scene::Editor => app.editor.render(f, root[1]),
        Scene::CalendarWeek => app.calendar.render_week(f, root[1]),
        Scene::MailCompose => {
            if let Some(ref mut compose) = app.compose {
                compose.render(f, root[1]);
            }
        }
        Scene::MailInbox => {
            app.inbox.render(f, root[1]);
        }
        Scene::DriveBrowser => {
            app.drive.render(f, root[1]);
        }
    }

    // Status line
    app.status.render(f, root[2]);

    // Palette overlay (if open)
    if let Some(ref mut palette) = app.palette {
        palette.render(f, f.area());
    }
}

fn render_tabs(f: &mut Frame, area: Rect, scene: Scene) {
    let tabs = ["Editor", "Calendar", "Inbox", "Drive"];
    let selected = match scene {
        Scene::Editor => 0,
        Scene::CalendarWeek => 1,
        Scene::MailInbox => 2,
        Scene::DriveBrowser => 3,
        Scene::MailCompose => 2,
    };

    let mut spans = Vec::new();
    for (i, tab) in tabs.iter().enumerate() {
        let style = if i == selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        spans.push(Span::styled(format!(" {} ", tab), style));
        if i + 1 < tabs.len() {
            spans.push(Span::raw("│"));
        }
    }

    let bar = Paragraph::new(Line::from(spans));
    f.render_widget(bar, area);
}
