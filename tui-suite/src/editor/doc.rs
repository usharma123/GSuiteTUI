use ropey::Rope;

#[derive(Debug, Clone, Copy, Default)]
pub struct Cursor {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub enum EditOp {
    Insert { pos: usize, text: String },
    Delete { pos: usize, text: String },
}

impl EditOp {
    pub fn inverse(&self) -> EditOp {
        match self {
            EditOp::Insert { pos, text } => EditOp::Delete {
                pos: *pos,
                text: text.clone(),
            },
            EditOp::Delete { pos, text } => EditOp::Insert {
                pos: *pos,
                text: text.clone(),
            },
        }
    }
}

#[derive(Debug)]
pub struct Document {
    pub rope: Rope,
    pub cursor: Cursor,
    pub undo_stack: Vec<EditOp>,
    pub redo_stack: Vec<EditOp>,
}

impl Document {
    pub fn new() -> Self {
        Self {
            rope: Rope::new(),
            cursor: Cursor::default(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn from_str(s: &str) -> Self {
        Self {
            rope: Rope::from_str(s),
            cursor: Cursor::default(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn len_lines(&self) -> usize {
        self.rope.len_lines()
    }

    pub fn line(&self, idx: usize) -> ropey::RopeSlice<'_> {
        self.rope.line(idx)
    }

    pub fn line_len(&self, line: usize) -> usize {
        if line >= self.rope.len_lines() {
            return 0;
        }
        let l = self.rope.line(line);
        let s = l.to_string();
        s.trim_end_matches('\n').chars().count()
    }

    pub fn char_index(&self, line: usize, col: usize) -> usize {
        let line_start = self.rope.line_to_char(line);
        line_start + col
    }

    pub fn cursor_index(&self) -> usize {
        self.char_index(self.cursor.line, self.cursor.col)
    }

    pub fn insert_str(&mut self, pos: usize, text: &str) {
        self.rope.insert(pos, text);
        self.undo_stack.push(EditOp::Insert {
            pos,
            text: text.to_string(),
        });
        self.redo_stack.clear();
    }

    pub fn insert_char(&mut self, pos: usize, c: char) {
        self.rope.insert_char(pos, c);
        self.undo_stack.push(EditOp::Insert {
            pos,
            text: c.to_string(),
        });
        self.redo_stack.clear();
    }

    pub fn delete_range(&mut self, start: usize, end: usize) {
        if start >= end || end > self.rope.len_chars() {
            return;
        }
        let text: String = self.rope.slice(start..end).chars().collect();
        self.rope.remove(start..end);
        self.undo_stack.push(EditOp::Delete { pos: start, text });
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) -> bool {
        if let Some(op) = self.undo_stack.pop() {
            self.apply_op(&op.inverse());
            self.redo_stack.push(op);
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(op) = self.redo_stack.pop() {
            self.apply_op(&op);
            self.undo_stack.push(op);
            true
        } else {
            false
        }
    }

    fn apply_op(&mut self, op: &EditOp) {
        match op {
            EditOp::Insert { pos, text } => {
                self.rope.insert(*pos, text);
            }
            EditOp::Delete { pos, text } => {
                let end = *pos + text.chars().count();
                if end <= self.rope.len_chars() {
                    self.rope.remove(*pos..end);
                }
            }
        }
    }

    pub fn to_string(&self) -> String {
        self.rope.to_string()
    }

    pub fn clamp_cursor(&mut self) {
        let max_line = self.rope.len_lines().saturating_sub(1);
        if self.cursor.line > max_line {
            self.cursor.line = max_line;
        }
        let max_col = self.line_len(self.cursor.line);
        if self.cursor.col > max_col {
            self.cursor.col = max_col;
        }
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}
