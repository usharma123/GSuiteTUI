use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use crate::drive::DriveDoc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteCommand {
    SwitchToEditor,
    SwitchToCalendar,
    OpenInbox,
    ComposeEmail,
    OpenDriveDoc,
    Save,
    SyncCalendar,
    Undo,
    Redo,
    Quit,
    LoginGoogle,
    SetupCredentials,
}

impl PaletteCommand {
    pub fn label(&self) -> &'static str {
        match self {
            PaletteCommand::SwitchToEditor => "Editor",
            PaletteCommand::SwitchToCalendar => "Calendar",
            PaletteCommand::OpenInbox => "Inbox",
            PaletteCommand::ComposeEmail => "Compose Email",
            PaletteCommand::OpenDriveDoc => "Open Drive Doc",
            PaletteCommand::Save => "Save",
            PaletteCommand::SyncCalendar => "Sync Calendar",
            PaletteCommand::Undo => "Undo",
            PaletteCommand::Redo => "Redo",
            PaletteCommand::Quit => "Quit",
            PaletteCommand::LoginGoogle => "Login Google",
            PaletteCommand::SetupCredentials => "Setup Credentials",
        }
    }

    pub fn all() -> Vec<PaletteCommand> {
        vec![
            PaletteCommand::SwitchToEditor,
            PaletteCommand::SwitchToCalendar,
            PaletteCommand::OpenInbox,
            PaletteCommand::ComposeEmail,
            PaletteCommand::OpenDriveDoc,
            PaletteCommand::Save,
            PaletteCommand::SyncCalendar,
            PaletteCommand::Undo,
            PaletteCommand::Redo,
            PaletteCommand::LoginGoogle,
            PaletteCommand::SetupCredentials,
            PaletteCommand::Quit,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteMode {
    Commands,
    DriveSearch,
}

#[derive(Debug, Clone)]
pub struct PaletteState {
    pub mode: PaletteMode,
    pub query: String,
    pub filtered: Vec<PaletteCommand>,
    pub drive_results: Vec<DriveDoc>,
    pub loading: bool,
    pub error: Option<String>,
    pub list_state: ListState,
}

impl PaletteState {
    pub fn new() -> Self {
        let mut state = Self {
            mode: PaletteMode::Commands,
            query: String::new(),
            filtered: PaletteCommand::all(),
            drive_results: Vec::new(),
            loading: false,
            error: None,
            list_state: ListState::default(),
        };
        state.list_state.select(Some(0));
        state
    }

    pub fn filter(&mut self) {
        if self.mode != PaletteMode::Commands {
            return;
        }
        let query_lower = self.query.to_lowercase();
        self.filtered = PaletteCommand::all()
            .into_iter()
            .filter(|cmd| cmd.label().to_lowercase().contains(&query_lower))
            .collect();

        // Reset selection
        if self.filtered.is_empty() {
            self.list_state.select(None);
        } else {
            self.list_state.select(Some(0));
        }
    }

    pub fn type_char(&mut self, c: char) {
        self.query.push(c);
        self.filter();
    }

    pub fn backspace(&mut self) {
        self.query.pop();
        self.filter();
    }

    pub fn move_up(&mut self) {
        match self.mode {
            PaletteMode::Commands => {
                if self.filtered.is_empty() {
                    return;
                }
                let i = match self.list_state.selected() {
                    Some(i) if i > 0 => i - 1,
                    Some(_) => self.filtered.len() - 1,
                    None => 0,
                };
                self.list_state.select(Some(i));
            }
            PaletteMode::DriveSearch => {
                if self.drive_results.is_empty() {
                    return;
                }
                let i = match self.list_state.selected() {
                    Some(i) if i > 0 => i - 1,
                    Some(_) => self.drive_results.len() - 1,
                    None => 0,
                };
                self.list_state.select(Some(i));
            }
        }
    }

    pub fn move_down(&mut self) {
        match self.mode {
            PaletteMode::Commands => {
                if self.filtered.is_empty() {
                    return;
                }
                let i = match self.list_state.selected() {
                    Some(i) if i < self.filtered.len() - 1 => i + 1,
                    Some(_) => 0,
                    None => 0,
                };
                self.list_state.select(Some(i));
            }
            PaletteMode::DriveSearch => {
                if self.drive_results.is_empty() {
                    return;
                }
                let i = match self.list_state.selected() {
                    Some(i) if i < self.drive_results.len() - 1 => i + 1,
                    Some(_) => 0,
                    None => 0,
                };
                self.list_state.select(Some(i));
            }
        }
    }

    pub fn selected_command(&self) -> Option<PaletteCommand> {
        self.list_state
            .selected()
            .and_then(|i| self.filtered.get(i).copied())
    }

    pub fn selected_drive_doc(&self) -> Option<DriveDoc> {
        self.list_state
            .selected()
            .and_then(|i| self.drive_results.get(i).cloned())
    }

    pub fn enter_drive_search(&mut self) {
        self.mode = PaletteMode::DriveSearch;
        self.query.clear();
        self.drive_results.clear();
        self.loading = true;
        self.error = None;
        self.list_state.select(Some(0));
    }

    pub fn enter_commands(&mut self) {
        self.mode = PaletteMode::Commands;
        self.query.clear();
        self.filtered = PaletteCommand::all();
        self.drive_results.clear();
        self.loading = false;
        self.error = None;
        self.list_state.select(Some(0));
    }

    pub fn set_drive_results(&mut self, results: Vec<DriveDoc>) {
        self.drive_results = results;
        self.loading = false;
        self.error = None;
        if self.drive_results.is_empty() {
            self.list_state.select(None);
        } else {
            self.list_state.select(Some(0));
        }
    }

    pub fn set_drive_loading(&mut self) {
        self.loading = true;
        self.error = None;
        if self.drive_results.is_empty() {
            self.list_state.select(None);
        }
    }

    pub fn set_drive_error(&mut self, message: String) {
        self.loading = false;
        self.error = Some(message);
    }

    pub fn is_drive_search(&self) -> bool {
        self.mode == PaletteMode::DriveSearch
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        // Center the palette
        let popup_area = centered_rect(60, 50, area);

        // Clear the area
        f.render_widget(Clear, popup_area);

        // Split into input and list
        let chunks = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(popup_area);

        // Input field
        let title = match self.mode {
            PaletteMode::Commands => "Command Palette",
            PaletteMode::DriveSearch => "Search Drive Docs",
        };
        let input = Paragraph::new(self.query.clone())
            .block(Block::default().title(title).borders(Borders::ALL));
        f.render_widget(input, chunks[0]);

        // Cursor in input
        f.set_cursor_position((chunks[0].x + 1 + self.query.len() as u16, chunks[0].y + 1));

        // Command list
        let items: Vec<ListItem> = match self.mode {
            PaletteMode::Commands => self
                .filtered
                .iter()
                .map(|cmd| ListItem::new(cmd.label()))
                .collect(),
            PaletteMode::DriveSearch => {
                if let Some(ref error) = self.error {
                    vec![ListItem::new(error.clone())]
                } else if self.loading && self.drive_results.is_empty() {
                    vec![ListItem::new("Loading...")]
                } else if self.drive_results.is_empty() {
                    vec![ListItem::new("No documents found")]
                } else {
                    self.drive_results
                        .iter()
                        .map(|doc| ListItem::new(doc.name.clone()))
                        .collect()
                }
            }
        };

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("> ");

        f.render_stateful_widget(list, chunks[1], &mut self.list_state);
    }
}

impl Default for PaletteState {
    fn default() -> Self {
        Self::new()
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
