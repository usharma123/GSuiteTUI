use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

#[derive(Debug, Clone)]
pub struct StatusLine {
    pub message: String,
}

impl StatusLine {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let paragraph = Paragraph::new(self.message.clone())
            .block(Block::default().borders(Borders::TOP));
        f.render_widget(paragraph, area);
    }
}

impl Default for StatusLine {
    fn default() -> Self {
        Self::new("")
    }
}
