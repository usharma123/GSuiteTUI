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
            Scene::Editor => {
                "Ctrl+S: save | Alt+1/2/3: heading | Ctrl+R: sync | Alt+D: Drive | Tab: next view"
                    .to_string()
            }
            Scene::CalendarWeek => {
                "h/l: day | H/L: week | j/k: scroll | g: now | Ctrl+R or s: sync"
                    .to_string()
            }
            Scene::MailCompose => {
                "Tab: next field | Ctrl+D: send | Ctrl+R: sync | Alt+D: Drive".to_string()
            }
            Scene::MailInbox => {
                "r: refresh | Ctrl+R: sync | Alt+D: Drive | Tab: next view".to_string()
            }
            Scene::DriveBrowser => {
                "Type to search | Enter: open | Ctrl+N: new doc | Alt+D: Drive | Tab: next view"
                    .to_string()
            }
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
