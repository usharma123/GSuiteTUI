use std::fs;
use std::path::PathBuf;

use crate::error::Result;

use super::doc::Document;

pub struct EditorState {
    pub doc: Document,
    pub top_line: usize,
    pub path: PathBuf,
}

impl EditorState {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let content = fs::read_to_string(&path).unwrap_or_default();
        let doc = Document::from_str(&content);
        Self {
            doc,
            top_line: 0,
            path,
        }
    }

    pub fn save(&self) -> Result<()> {
        fs::write(&self.path, self.doc.to_string())?;
        Ok(())
    }

    // Cursor movement
    pub fn move_up(&mut self) {
        if self.doc.cursor.line > 0 {
            self.doc.cursor.line -= 1;
            self.doc.clamp_cursor();
        }
    }

    pub fn move_down(&mut self) {
        if self.doc.cursor.line + 1 < self.doc.len_lines() {
            self.doc.cursor.line += 1;
            self.doc.clamp_cursor();
        }
    }

    pub fn move_left(&mut self) {
        if self.doc.cursor.col > 0 {
            self.doc.cursor.col -= 1;
        } else if self.doc.cursor.line > 0 {
            self.doc.cursor.line -= 1;
            self.doc.cursor.col = self.doc.line_len(self.doc.cursor.line);
        }
    }

    pub fn move_right(&mut self) {
        let max_col = self.doc.line_len(self.doc.cursor.line);
        if self.doc.cursor.col < max_col {
            self.doc.cursor.col += 1;
        } else if self.doc.cursor.line + 1 < self.doc.len_lines() {
            self.doc.cursor.line += 1;
            self.doc.cursor.col = 0;
        }
    }

    pub fn move_home(&mut self) {
        self.doc.cursor.col = 0;
    }

    pub fn move_end(&mut self) {
        self.doc.cursor.col = self.doc.line_len(self.doc.cursor.line);
    }

    // Editing
    pub fn insert_char(&mut self, c: char) {
        let pos = self.doc.cursor_index();
        self.doc.insert_char(pos, c);
        self.doc.cursor.col += 1;
    }

    pub fn insert_newline(&mut self) {
        let pos = self.doc.cursor_index();
        self.doc.insert_char(pos, '\n');
        self.doc.cursor.line += 1;
        self.doc.cursor.col = 0;
    }

    pub fn backspace(&mut self) {
        let pos = self.doc.cursor_index();
        if pos == 0 {
            return;
        }
        let prev = pos - 1;
        let ch = self.doc.rope.char(prev);
        self.doc.delete_range(prev, pos);

        if ch == '\n' {
            self.doc.cursor.line = self.doc.cursor.line.saturating_sub(1);
            self.doc.cursor.col = self.doc.line_len(self.doc.cursor.line);
        } else {
            self.doc.cursor.col = self.doc.cursor.col.saturating_sub(1);
        }
    }

    pub fn delete(&mut self) {
        let pos = self.doc.cursor_index();
        if pos < self.doc.rope.len_chars() {
            self.doc.delete_range(pos, pos + 1);
        }
    }

    // Undo/Redo
    pub fn undo(&mut self) {
        self.doc.undo();
        self.doc.clamp_cursor();
    }

    pub fn redo(&mut self) {
        self.doc.redo();
        self.doc.clamp_cursor();
    }

    // Markdown formatting
    pub fn toggle_bold(&mut self) {
        self.insert_str_at_cursor("****");
        self.doc.cursor.col += 2;
    }

    pub fn toggle_italic(&mut self) {
        self.insert_str_at_cursor("**");
        self.doc.cursor.col += 1;
    }

    pub fn insert_heading(&mut self, level: u8) {
        // Move to start of line
        let line_start = self.doc.char_index(self.doc.cursor.line, 0);
        let prefix = "#".repeat(level as usize) + " ";
        self.doc.insert_str(line_start, &prefix);
        self.doc.cursor.col += prefix.len();
    }

    fn insert_str_at_cursor(&mut self, s: &str) {
        let pos = self.doc.cursor_index();
        self.doc.insert_str(pos, s);
    }

    // Rendering helpers
    pub fn ensure_cursor_visible(&mut self, height: usize) {
        if self.doc.cursor.line < self.top_line {
            self.top_line = self.doc.cursor.line;
        } else if self.doc.cursor.line >= self.top_line + height {
            self.top_line = self.doc.cursor.line.saturating_sub(height - 1);
        }
    }
}
