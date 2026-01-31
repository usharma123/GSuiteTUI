use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteCommand {
    SwitchToEditor,
    SwitchToCalendar,
    OpenInbox,
    ComposeEmail,
    Save,
    SyncCalendar,
    Undo,
    Redo,
    Quit,
    LoginGoogle,
}

impl PaletteCommand {
    pub fn label(&self) -> &'static str {
        match self {
            PaletteCommand::SwitchToEditor => "Editor",
            PaletteCommand::SwitchToCalendar => "Calendar",
            PaletteCommand::OpenInbox => "Inbox",
            PaletteCommand::ComposeEmail => "Compose Email",
            PaletteCommand::Save => "Save",
            PaletteCommand::SyncCalendar => "Sync Calendar",
            PaletteCommand::Undo => "Undo",
            PaletteCommand::Redo => "Redo",
            PaletteCommand::Quit => "Quit",
            PaletteCommand::LoginGoogle => "Login Google",
        }
    }

    pub fn all() -> Vec<PaletteCommand> {
        vec![
            PaletteCommand::SwitchToEditor,
            PaletteCommand::SwitchToCalendar,
            PaletteCommand::OpenInbox,
            PaletteCommand::ComposeEmail,
            PaletteCommand::Save,
            PaletteCommand::SyncCalendar,
            PaletteCommand::Undo,
            PaletteCommand::Redo,
            PaletteCommand::LoginGoogle,
            PaletteCommand::Quit,
        ]
    }
}

#[derive(Debug, Clone)]
pub struct PaletteState {
    pub query: String,
    pub filtered: Vec<PaletteCommand>,
    pub list_state: ListState,
}

impl PaletteState {
    pub fn new() -> Self {
        let mut state = Self {
            query: String::new(),
            filtered: PaletteCommand::all(),
            list_state: ListState::default(),
        };
        state.list_state.select(Some(0));
        state
    }

    pub fn filter(&mut self) {
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

    pub fn move_down(&mut self) {
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

    pub fn selected_command(&self) -> Option<PaletteCommand> {
        self.list_state
            .selected()
            .and_then(|i| self.filtered.get(i).copied())
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
        let input = Paragraph::new(self.query.clone())
            .block(Block::default().title("Command Palette").borders(Borders::ALL));
        f.render_widget(input, chunks[0]);

        // Cursor in input
        f.set_cursor_position((chunks[0].x + 1 + self.query.len() as u16, chunks[0].y + 1));

        // Command list
        let items: Vec<ListItem> = self
            .filtered
            .iter()
            .map(|cmd| ListItem::new(cmd.label()))
            .collect();

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
