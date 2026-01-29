use ratatui::layout::Rect;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use super::ops::EditorState;

impl EditorState {
    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Editor (Markdown)")
            .borders(Borders::ALL);
        let inner = block.inner(area);
        f.render_widget(block, area);

        let height = inner.height as usize;
        self.ensure_cursor_visible(height);

        // Render visible lines
        let mut text = Text::default();
        for i in 0..height {
            let line_idx = self.top_line + i;
            if line_idx >= self.doc.len_lines() {
                break;
            }
            let mut s = self.doc.line(line_idx).to_string();
            s = s.trim_end_matches('\n').to_string();
            text.lines.push(Line::from(s));
        }

        let paragraph = Paragraph::new(text);
        f.render_widget(paragraph, inner);

        // Cursor position
        let cursor_y = (self.doc.cursor.line.saturating_sub(self.top_line)) as u16;
        let cursor_x = self.doc.cursor.col as u16;
        if cursor_y < inner.height && cursor_x < inner.width {
            f.set_cursor_position((inner.x + cursor_x, inner.y + cursor_y));
        }
    }
}
