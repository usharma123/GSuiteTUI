use std::fs;
use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::drive::DriveDoc;

use super::doc::Document;

#[derive(Debug, Clone)]
pub enum EditorSource {
    Local { path: PathBuf },
    DriveDoc { doc: DriveDoc },
}

pub struct EditorState {
    pub doc: Document,
    pub top_line: usize,
    pub source: EditorSource,
    pub modified: bool,
}

impl EditorState {
    pub fn new_local(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let content = fs::read_to_string(&path).unwrap_or_default();
        let doc = Document::from_str(&content);
        Self {
            doc,
            top_line: 0,
            source: EditorSource::Local { path },
            modified: false,
        }
    }

    pub fn new_drive(doc: DriveDoc, content: &str) -> Self {
        let doc_state = Document::from_str(content);
        Self {
            doc: doc_state,
            top_line: 0,
            source: EditorSource::DriveDoc { doc },
            modified: false,
        }
    }

    pub fn save_local(&mut self) -> Result<()> {
        let path = match &self.source {
            EditorSource::Local { path } => path,
            EditorSource::DriveDoc { .. } => {
                return Ok(());
            }
        };
        fs::write(path, self.doc.to_string())?;
        self.modified = false;
        Ok(())
    }
    
    /// Check if the document has unsaved changes
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    pub fn set_saved(&mut self) {
        self.modified = false;
    }
    
    /// Mark the document as modified
    fn mark_modified(&mut self) {
        self.modified = true;
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
        self.mark_modified();
    }

    pub fn insert_newline(&mut self) {
        let pos = self.doc.cursor_index();
        self.doc.insert_char(pos, '\n');
        self.doc.cursor.line += 1;
        self.doc.cursor.col = 0;
        self.mark_modified();
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
        self.mark_modified();
    }

    pub fn delete(&mut self) {
        let pos = self.doc.cursor_index();
        if pos < self.doc.rope.len_chars() {
            self.doc.delete_range(pos, pos + 1);
            self.mark_modified();
        }
    }

    // Undo/Redo
    pub fn undo(&mut self) {
        self.doc.undo();
        self.doc.clamp_cursor();
        self.mark_modified();
    }

    pub fn redo(&mut self) {
        self.doc.redo();
        self.doc.clamp_cursor();
        self.mark_modified();
    }

    // Markdown formatting
    pub fn toggle_bold(&mut self) {
        self.insert_str_at_cursor("****");
        self.doc.cursor.col += 2;
        self.mark_modified();
    }

    pub fn toggle_italic(&mut self) {
        self.insert_str_at_cursor("**");
        self.doc.cursor.col += 1;
        self.mark_modified();
    }

    pub fn insert_heading(&mut self, level: u8) {
        // Move to start of line
        let line_start = self.doc.char_index(self.doc.cursor.line, 0);
        let prefix = "#".repeat(level as usize) + " ";
        self.doc.insert_str(line_start, &prefix);
        self.doc.cursor.col += prefix.len();
        self.mark_modified();
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

    pub fn source_title(&self) -> String {
        match &self.source {
            EditorSource::Local { path } => {
                path.file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("notes.md")
                    .to_string()
            }
            EditorSource::DriveDoc { doc } => doc.name.clone(),
        }
    }

    pub fn drive_doc(&self) -> Option<DriveDoc> {
        match &self.source {
            EditorSource::DriveDoc { doc } => Some(doc.clone()),
            _ => None,
        }
    }

    pub fn markdown(&self) -> String {
        self.doc.to_string()
    }

    pub fn replace_with_drive_doc(&mut self, doc: DriveDoc, markdown: &str) {
        self.doc = Document::from_str(markdown);
        self.top_line = 0;
        self.source = EditorSource::DriveDoc { doc };
        self.modified = false;
    }

    pub fn replace_with_local_file(&mut self, path: impl Into<PathBuf>) -> Result<()> {
        let path = path.into();
        let content = fs::read_to_string(&path).unwrap_or_default();
        self.doc = Document::from_str(&content);
        self.top_line = 0;
        self.source = EditorSource::Local { path };
        self.modified = false;
        Ok(())
    }

    pub fn local_path(&self) -> Option<&Path> {
        match &self.source {
            EditorSource::Local { path } => Some(path.as_path()),
            _ => None,
        }
    }
}
