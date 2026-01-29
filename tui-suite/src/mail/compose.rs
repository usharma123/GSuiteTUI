use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::editor::doc::Document;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComposeField {
    To,
    Subject,
    Body,
}

pub struct ComposeState {
    pub to: String,
    pub subject: String,
    pub body: Document,
    pub focus: ComposeField,
    pub cursor_pos: usize,
    pub top_line: usize,
}

impl ComposeState {
    pub fn new() -> Self {
        Self {
            to: String::new(),
            subject: String::new(),
            body: Document::new(),
            focus: ComposeField::To,
            cursor_pos: 0,
            top_line: 0,
        }
    }

    pub fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            ComposeField::To => ComposeField::Subject,
            ComposeField::Subject => ComposeField::Body,
            ComposeField::Body => ComposeField::To,
        };
        self.cursor_pos = match self.focus {
            ComposeField::To => self.to.len(),
            ComposeField::Subject => self.subject.len(),
            ComposeField::Body => 0,
        };
    }

    pub fn insert_char(&mut self, c: char) {
        match self.focus {
            ComposeField::To => {
                self.to.insert(self.cursor_pos, c);
                self.cursor_pos += 1;
            }
            ComposeField::Subject => {
                self.subject.insert(self.cursor_pos, c);
                self.cursor_pos += 1;
            }
            ComposeField::Body => {
                let pos = self.body.cursor_index();
                self.body.insert_char(pos, c);
                self.body.cursor.col += 1;
            }
        }
    }

    pub fn backspace(&mut self) {
        match self.focus {
            ComposeField::To if self.cursor_pos > 0 => {
                self.cursor_pos -= 1;
                self.to.remove(self.cursor_pos);
            }
            ComposeField::Subject if self.cursor_pos > 0 => {
                self.cursor_pos -= 1;
                self.subject.remove(self.cursor_pos);
            }
            ComposeField::Body => {
                let pos = self.body.cursor_index();
                if pos > 0 {
                    let ch = self.body.rope.char(pos - 1);
                    self.body.delete_range(pos - 1, pos);
                    if ch == '\n' {
                        self.body.cursor.line = self.body.cursor.line.saturating_sub(1);
                        self.body.cursor.col = self.body.line_len(self.body.cursor.line);
                    } else {
                        self.body.cursor.col = self.body.cursor.col.saturating_sub(1);
                    }
                }
            }
            _ => {}
        }
    }

    pub fn insert_newline(&mut self) {
        if self.focus == ComposeField::Body {
            let pos = self.body.cursor_index();
            self.body.insert_char(pos, '\n');
            self.body.cursor.line += 1;
            self.body.cursor.col = 0;
        }
    }

    pub fn move_left(&mut self) {
        match self.focus {
            ComposeField::To | ComposeField::Subject => {
                self.cursor_pos = self.cursor_pos.saturating_sub(1);
            }
            ComposeField::Body => {
                if self.body.cursor.col > 0 {
                    self.body.cursor.col -= 1;
                } else if self.body.cursor.line > 0 {
                    self.body.cursor.line -= 1;
                    self.body.cursor.col = self.body.line_len(self.body.cursor.line);
                }
            }
        }
    }

    pub fn move_right(&mut self) {
        match self.focus {
            ComposeField::To => {
                if self.cursor_pos < self.to.len() {
                    self.cursor_pos += 1;
                }
            }
            ComposeField::Subject => {
                if self.cursor_pos < self.subject.len() {
                    self.cursor_pos += 1;
                }
            }
            ComposeField::Body => {
                let max_col = self.body.line_len(self.body.cursor.line);
                if self.body.cursor.col < max_col {
                    self.body.cursor.col += 1;
                } else if self.body.cursor.line + 1 < self.body.len_lines() {
                    self.body.cursor.line += 1;
                    self.body.cursor.col = 0;
                }
            }
        }
    }

    pub fn move_up(&mut self) {
        if self.focus == ComposeField::Body && self.body.cursor.line > 0 {
            self.body.cursor.line -= 1;
            self.body.clamp_cursor();
        }
    }

    pub fn move_down(&mut self) {
        if self.focus == ComposeField::Body && self.body.cursor.line + 1 < self.body.len_lines() {
            self.body.cursor.line += 1;
            self.body.clamp_cursor();
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Compose Email (Ctrl+Enter to send)")
            .borders(Borders::ALL);
        let inner = block.inner(area);
        f.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // To
                Constraint::Length(3), // Subject
                Constraint::Min(1),    // Body
            ])
            .split(inner);

        // To field
        let to_style = if self.focus == ComposeField::To {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let to_para = Paragraph::new(self.to.clone())
            .block(Block::default().title("To:").borders(Borders::ALL))
            .style(to_style);
        f.render_widget(to_para, chunks[0]);

        // Subject field
        let subject_style = if self.focus == ComposeField::Subject {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let subject_para = Paragraph::new(self.subject.clone())
            .block(Block::default().title("Subject:").borders(Borders::ALL))
            .style(subject_style);
        f.render_widget(subject_para, chunks[1]);

        // Body field
        let body_style = if self.focus == ComposeField::Body {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let body_block = Block::default().title("Body:").borders(Borders::ALL);
        let body_inner = body_block.inner(chunks[2]);
        f.render_widget(body_block.style(body_style), chunks[2]);

        // Render body text
        let height = body_inner.height as usize;
        if self.body.cursor.line < self.top_line {
            self.top_line = self.body.cursor.line;
        } else if self.body.cursor.line >= self.top_line + height {
            self.top_line = self.body.cursor.line.saturating_sub(height - 1);
        }

        let mut text = Text::default();
        for i in 0..height {
            let line_idx = self.top_line + i;
            if line_idx >= self.body.len_lines() {
                break;
            }
            let mut s = self.body.line(line_idx).to_string();
            s = s.trim_end_matches('\n').to_string();
            text.lines.push(Line::from(s));
        }
        let body_para = Paragraph::new(text);
        f.render_widget(body_para, body_inner);

        // Set cursor position
        match self.focus {
            ComposeField::To => {
                f.set_cursor_position((
                    chunks[0].x + 1 + self.cursor_pos as u16,
                    chunks[0].y + 1,
                ));
            }
            ComposeField::Subject => {
                f.set_cursor_position((
                    chunks[1].x + 1 + self.cursor_pos as u16,
                    chunks[1].y + 1,
                ));
            }
            ComposeField::Body => {
                let cursor_y = (self.body.cursor.line.saturating_sub(self.top_line)) as u16;
                let cursor_x = self.body.cursor.col as u16;
                if cursor_y < body_inner.height && cursor_x < body_inner.width {
                    f.set_cursor_position((body_inner.x + cursor_x, body_inner.y + cursor_y));
                }
            }
        }
    }
}

impl Default for ComposeState {
    fn default() -> Self {
        Self::new()
    }
}
