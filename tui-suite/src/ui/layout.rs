use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::{AppState, Scene};
use crate::sheets::view::render_sheets;

pub fn render_app(f: &mut Frame, app: &mut AppState) {
    // Setup wizard is fullscreen overlay
    if app.scene == Scene::Setup {
        if let Some(ref setup) = app.setup {
            setup.render(f, f.area());
        }
        return;
    }

    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(f.area());

    render_tabs(f, root[0], app.scene);

    let shortcuts_hint = app.status.hint.clone();

    // Main content area
    match app.scene {
        Scene::Editor => app.editor.render(f, root[1]),
        Scene::CalendarWeek => render_with_shortcuts(f, root[1], &shortcuts_hint, |f, area| {
            app.calendar.render_week(f, area);
        }),
        Scene::MailCompose => {
            if let Some(ref mut compose) = app.compose {
                render_with_shortcuts(f, root[1], &shortcuts_hint, |f, area| {
                    compose.render(f, area);
                });
            }
        }
        Scene::MailInbox => {
            render_with_shortcuts(f, root[1], &shortcuts_hint, |f, area| {
                app.inbox.render(f, area);
            });
        }
        Scene::DriveBrowser => {
            render_with_shortcuts(f, root[1], &shortcuts_hint, |f, area| {
                app.drive.render(f, area);
            });
        }
        Scene::Sheets => {
            render_with_shortcuts(f, root[1], &shortcuts_hint, |f, area| {
                render_sheets(f, area, &mut app.sheets);
            });
        }
        Scene::Setup => {} // Handled above
    }

    // Status line
    app.status.render(f, root[2]);

    // Palette overlay (if open)
    if let Some(ref mut palette) = app.palette {
        palette.render(f, f.area());
    }
}

fn render_tabs(f: &mut Frame, area: Rect, scene: Scene) {
    let tabs = ["Editor", "Calendar", "Inbox", "Drive", "Sheets"];
    let selected = match scene {
        Scene::Editor => 0,
        Scene::CalendarWeek => 1,
        Scene::MailInbox => 2,
        Scene::DriveBrowser => 3,
        Scene::Sheets => 4,
        Scene::MailCompose => 2,
        Scene::Setup => 0, // Not shown, but needed for exhaustiveness
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

fn render_with_shortcuts<F>(f: &mut Frame, area: Rect, hint: &str, render_content: F)
where
    F: FnOnce(&mut Frame, Rect),
{
    if hint.is_empty() {
        render_content(f, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    render_content(f, chunks[0]);
    render_shortcuts_bar(f, chunks[1], hint);
}

fn render_shortcuts_bar(f: &mut Frame, area: Rect, hint: &str) {
    let bar = Paragraph::new(hint)
        .style(Style::default().fg(Color::Gray).bg(Color::Rgb(30, 30, 40)));
    f.render_widget(bar, area);
}
