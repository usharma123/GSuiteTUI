use chrono::{DateTime, Utc};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Frame;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailSummary {
    pub id: String,
    pub thread_id: String,
    pub from: String,
    pub subject: String,
    pub snippet: String,
    pub date: DateTime<Utc>,
    pub is_unread: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailDetail {
    pub id: String,
    pub from: String,
    pub to: String,
    pub subject: String,
    pub date: DateTime<Utc>,
    pub body_text: String,
    pub body_html: Option<String>,
}

#[derive(Debug)]
pub struct InboxState {
    pub emails: Vec<EmailSummary>,
    pub selected: usize,
    pub list_state: ListState,
    pub current_email: Option<EmailDetail>,
    pub scroll_offset: u16,
    pub view_mode: InboxViewMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InboxViewMode {
    List,
    Reading,
}

impl InboxState {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            emails: Vec::new(),
            selected: 0,
            list_state,
            current_email: None,
            scroll_offset: 0,
            view_mode: InboxViewMode::List,
        }
    }

    pub fn select_next(&mut self) {
        if self.emails.is_empty() {
            return;
        }
        self.selected = (self.selected + 1).min(self.emails.len() - 1);
        self.list_state.select(Some(self.selected));
    }

    pub fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
        self.list_state.select(Some(self.selected));
    }

    pub fn selected_email(&self) -> Option<&EmailSummary> {
        self.emails.get(self.selected)
    }

    pub fn open_email(&mut self, detail: EmailDetail) {
        self.current_email = Some(detail);
        self.view_mode = InboxViewMode::Reading;
        self.scroll_offset = 0;
    }

    pub fn close_email(&mut self) {
        self.current_email = None;
        self.view_mode = InboxViewMode::List;
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(1);
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        match self.view_mode {
            InboxViewMode::List => self.render_list(f, area),
            InboxViewMode::Reading => self.render_email(f, area),
        }
    }

    fn render_list(&mut self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(format!("Inbox ({} emails) - j/k: navigate, Enter: open, r: refresh", self.emails.len()))
            .borders(Borders::ALL);
        let inner = block.inner(area);
        f.render_widget(block, area);

        if self.emails.is_empty() {
            let msg = Paragraph::new("No emails. Press 'r' to refresh.")
                .alignment(Alignment::Center);
            f.render_widget(msg, inner);
            return;
        }

        let items: Vec<ListItem> = self
            .emails
            .iter()
            .enumerate()
            .map(|(i, email)| {
                let unread_marker = if email.is_unread { "●" } else { " " };
                let date_str = email.date.format("%b %d").to_string();

                // Truncate from/subject to fit
                let from = if email.from.len() > 20 {
                    format!("{}…", &email.from[..19])
                } else {
                    format!("{:20}", email.from)
                };

                let line = format!(
                    "{} {} │ {} │ {}",
                    unread_marker,
                    date_str,
                    from,
                    email.subject
                );

                let style = if i == self.selected {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else if email.is_unread {
                    Style::default().add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Gray)
                };

                ListItem::new(line).style(style)
            })
            .collect();

        let list = List::new(items);
        f.render_stateful_widget(list, inner, &mut self.list_state);
    }

    fn render_email(&mut self, f: &mut Frame, area: Rect) {
        let email = match &self.current_email {
            Some(e) => e,
            None => {
                self.view_mode = InboxViewMode::List;
                return;
            }
        };

        let block = Block::default()
            .title("Reading Email - q/Esc: back, j/k: scroll")
            .borders(Borders::ALL);
        let inner = block.inner(area);
        f.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4), // Headers
                Constraint::Min(1),    // Body
            ])
            .split(inner);

        // Headers
        let headers = format!(
            "From: {}\nTo: {}\nDate: {}\nSubject: {}",
            email.from,
            email.to,
            email.date.format("%Y-%m-%d %H:%M"),
            email.subject
        );
        let headers_widget = Paragraph::new(headers)
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default().borders(Borders::BOTTOM));
        f.render_widget(headers_widget, chunks[0]);

        // Body with scroll
        let body_lines: Vec<&str> = email.body_text.lines().collect();
        let visible_lines: String = body_lines
            .iter()
            .skip(self.scroll_offset as usize)
            .take(chunks[1].height as usize)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");

        let body_widget = Paragraph::new(visible_lines)
            .wrap(Wrap { trim: false });
        f.render_widget(body_widget, chunks[1]);
    }
}

impl Default for InboxState {
    fn default() -> Self {
        Self::new()
    }
}
