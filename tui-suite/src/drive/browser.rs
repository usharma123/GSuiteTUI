use chrono::Local;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use super::DriveDoc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriveBrowserMode {
    Browsing,
    Creating,
}

#[derive(Debug, Clone)]
pub struct DriveBrowserState {
    pub docs: Vec<DriveDoc>,
    pub list_state: ListState,
    pub query: String,
    pub loading: bool,
    pub error: Option<String>,
    pub mode: DriveBrowserMode,
    pub new_doc_name: String,
}

impl DriveBrowserState {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            docs: Vec::new(),
            list_state,
            query: String::new(),
            loading: false,
            error: None,
            mode: DriveBrowserMode::Browsing,
            new_doc_name: String::new(),
        }
    }

    pub fn reset(&mut self) {
        self.docs.clear();
        self.query.clear();
        self.loading = true;
        self.error = None;
        self.mode = DriveBrowserMode::Browsing;
        self.new_doc_name.clear();
        self.list_state.select(Some(0));
    }

    pub fn move_up(&mut self) {
        if self.docs.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) if i > 0 => i - 1,
            Some(_) => self.docs.len() - 1,
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn move_down(&mut self) {
        if self.docs.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) if i + 1 < self.docs.len() => i + 1,
            Some(_) => 0,
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn selected_doc(&self) -> Option<DriveDoc> {
        self.list_state
            .selected()
            .and_then(|i| self.docs.get(i).cloned())
    }

    pub fn type_char(&mut self, c: char) {
        match self.mode {
            DriveBrowserMode::Browsing => self.query.push(c),
            DriveBrowserMode::Creating => self.new_doc_name.push(c),
        }
    }

    pub fn backspace(&mut self) {
        match self.mode {
            DriveBrowserMode::Browsing => {
                self.query.pop();
            }
            DriveBrowserMode::Creating => {
                self.new_doc_name.pop();
            }
        }
    }

    pub fn clear_query(&mut self) {
        self.query.clear();
    }

    pub fn start_create(&mut self) {
        self.mode = DriveBrowserMode::Creating;
        self.new_doc_name.clear();
    }

    pub fn cancel_create(&mut self) {
        self.mode = DriveBrowserMode::Browsing;
        self.new_doc_name.clear();
    }

    pub fn is_creating(&self) -> bool {
        self.mode == DriveBrowserMode::Creating
    }

    pub fn set_docs(&mut self, docs: Vec<DriveDoc>) {
        self.docs = docs;
        self.loading = false;
        self.error = None;
        if self.docs.is_empty() {
            self.list_state.select(None);
        } else {
            self.list_state.select(Some(0));
        }
    }

    pub fn set_loading(&mut self) {
        self.loading = true;
        self.error = None;
        if self.docs.is_empty() {
            self.list_state.select(None);
        }
    }

    pub fn set_error(&mut self, message: String) {
        self.loading = false;
        self.error = Some(message);
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(area);

        let search_title = "Drive Search (type to filter)";
        let search = Paragraph::new(self.query.clone())
            .block(Block::default().title(search_title).borders(Borders::ALL));
        f.render_widget(search, chunks[0]);

        if !self.is_creating() {
            f.set_cursor_position((
                chunks[0].x + 1 + self.query.len() as u16,
                chunks[0].y + 1,
            ));
        }

        let items: Vec<ListItem> = if let Some(ref error) = self.error {
            vec![ListItem::new(error.clone())]
        } else if self.loading && self.docs.is_empty() {
            vec![ListItem::new("Loading...")]
        } else if self.docs.is_empty() {
            vec![ListItem::new("No documents found")]
        } else {
            self.docs
                .iter()
                .map(|doc| {
                    let date = doc
                        .modified_time
                        .map(|dt| dt.with_timezone(&Local).format("%Y-%m-%d").to_string())
                        .unwrap_or_else(|| "-".to_string());
                    let line = format!("{:12}  {}", date, doc.name);
                    ListItem::new(line)
                })
                .collect()
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .title("Drive Docs (Enter: open, Ctrl+N: new)")
                    .borders(Borders::ALL),
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("> ");
        f.render_stateful_widget(list, chunks[1], &mut self.list_state);

        if self.is_creating() {
            let popup_area = centered_rect(60, 30, area);
            f.render_widget(Clear, popup_area);
            let title = "New Drive Doc (Enter: create, Esc: cancel)";
            let input = Paragraph::new(self.new_doc_name.clone())
                .block(Block::default().title(title).borders(Borders::ALL));
            f.render_widget(input, popup_area);
            f.set_cursor_position((
                popup_area.x + 1 + self.new_doc_name.len() as u16,
                popup_area.y + 1,
            ));
        }
    }
}

impl Default for DriveBrowserState {
    fn default() -> Self {
        Self::new()
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
