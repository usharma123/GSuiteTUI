use ratatui::layout::Rect;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::Scene;

#[derive(Debug, Clone)]
pub struct StatusLine {
    pub message: String,
    pub hint: String,
}

impl StatusLine {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            hint: String::new(),
        }
    }

    pub fn set_hint_for_scene(&mut self, scene: Scene) {
        self.hint = match scene {
            Scene::Editor => "Ctrl+S: save | Ctrl+1/2/3: heading | Tab: next view".to_string(),
            Scene::CalendarWeek => "h/l: day | H/L: week | j/k: scroll | g: now | Enter: details".to_string(),
            Scene::MailCompose => "Tab: next field | Ctrl+Enter: send | q: quit".to_string(),
            Scene::MailInbox => "Tab: next view | q: quit".to_string(),
        };
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        // Combine message and hint
        let display = if self.message.is_empty() {
            self.hint.clone()
        } else if self.hint.is_empty() {
            self.message.clone()
        } else {
            format!("{} │ {}", self.message, self.hint)
        };

        let paragraph = Paragraph::new(display)
            .block(Block::default().borders(Borders::TOP))
            .style(Style::default().fg(Color::Gray));
        f.render_widget(paragraph, area);
    }
}

impl Default for StatusLine {
    fn default() -> Self {
        Self::new("")
    }
}
