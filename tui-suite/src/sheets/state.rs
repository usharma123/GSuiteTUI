use chrono::{DateTime, Utc};
use ratatui::widgets::ListState;

use super::provider::{SheetData, SheetTab, SpreadsheetDoc, SpreadsheetMeta};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SheetsViewMode {
    Browser,
    Grid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SheetMode {
    View,
    Edit,
}

#[derive(Debug, Clone)]
pub struct SheetsBrowserState {
    pub docs: Vec<SpreadsheetDoc>,
    pub list_state: ListState,
    pub query: String,
    pub loading: bool,
    pub error: Option<String>,
}

impl SheetsBrowserState {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            docs: Vec::new(),
            list_state,
            query: String::new(),
            loading: false,
            error: None,
        }
    }

    pub fn reset(&mut self) {
        self.docs.clear();
        self.query.clear();
        self.loading = true;
        self.error = None;
        self.list_state.select(Some(0));
    }

    pub fn move_up(&mut self) {
        if self.docs.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) if i > 0 => i - 1,
            Some(_) => self.docs.len() - 1,
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn move_down(&mut self) {
        if self.docs.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) if i + 1 < self.docs.len() => i + 1,
            Some(_) => 0,
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn selected_doc(&self) -> Option<SpreadsheetDoc> {
        self.list_state
            .selected()
            .and_then(|i| self.docs.get(i).cloned())
    }

    pub fn type_char(&mut self, c: char) {
        self.query.push(c);
    }

    pub fn backspace(&mut self) {
        self.query.pop();
    }

    pub fn set_docs(&mut self, docs: Vec<SpreadsheetDoc>) {
        self.docs = docs;
        self.loading = false;
        self.error = None;
        if self.docs.is_empty() {
            self.list_state.select(None);
        } else {
            self.list_state.select(Some(0));
        }
    }

    pub fn set_loading(&mut self) {
        self.loading = true;
        self.error = None;
        if self.docs.is_empty() {
            self.list_state.select(None);
        }
    }

    pub fn set_error(&mut self, message: String) {
        self.loading = false;
        self.error = Some(message);
    }
}

impl Default for SheetsBrowserState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct SheetViewport {
    pub row: usize,
    pub col: usize,
    pub height: usize,
    pub width: usize,
}

impl SheetViewport {
    pub fn new() -> Self {
        Self {
            row: 0,
            col: 0,
            height: 0,
            width: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SheetCursor {
    pub row: usize,
    pub col: usize,
}

impl SheetCursor {
    pub fn new() -> Self {
        Self { row: 0, col: 0 }
    }
}

#[derive(Debug, Clone)]
pub struct UsedRange {
    pub rows: usize,
    pub cols: usize,
}

impl UsedRange {
    pub fn new() -> Self {
        Self { rows: 1, cols: 1 }
    }
}

#[derive(Debug, Clone)]
pub struct SheetGridState {
    pub spreadsheet_id: Option<String>,
    pub spreadsheet_name: Option<String>,
    pub tabs: Vec<SheetTab>,
    pub active_tab: usize,
    pub viewport: SheetViewport,
    pub cursor: SheetCursor,
    pub mode: SheetMode,
    pub edit_buffer: String,
    pub values: Vec<Vec<String>>,
    pub used_range: UsedRange,
}

impl SheetGridState {
    pub fn new() -> Self {
        Self {
            spreadsheet_id: None,
            spreadsheet_name: None,
            tabs: Vec::new(),
            active_tab: 0,
            viewport: SheetViewport::new(),
            cursor: SheetCursor::new(),
            mode: SheetMode::View,
            edit_buffer: String::new(),
            values: Vec::new(),
            used_range: UsedRange::new(),
        }
    }

    pub fn is_loaded(&self) -> bool {
        self.spreadsheet_id.is_some() && !self.tabs.is_empty()
    }

    pub fn active_sheet_name(&self) -> Option<String> {
        self.tabs.get(self.active_tab).map(|t| t.name.clone())
    }

    pub fn set_meta(&mut self, meta: SpreadsheetMeta) {
        self.spreadsheet_id = Some(meta.id);
        self.spreadsheet_name = Some(meta.name);
        self.tabs = meta.sheets;
        self.active_tab = 0;
        self.cursor = SheetCursor::new();
        self.viewport = SheetViewport::new();
        self.mode = SheetMode::View;
        self.edit_buffer.clear();
        self.values.clear();
        self.used_range = UsedRange::new();
    }

    pub fn set_values(&mut self, data: SheetData) {
        self.values = data.values;
        let rows = self.values.len().max(1);
        let cols = self
            .values
            .iter()
            .map(|row| row.len())
            .max()
            .unwrap_or(1)
            .max(1);
        self.used_range = UsedRange { rows, cols };
        self.ensure_cursor_in_bounds();
    }

    pub fn set_viewport_size(&mut self, height: usize, width: usize) {
        self.viewport.height = height;
        self.viewport.width = width;
        self.ensure_cursor_visible();
    }

    pub fn move_up(&mut self) {
        if self.cursor.row > 0 {
            self.cursor.row -= 1;
        }
        self.ensure_cursor_visible();
    }

    pub fn move_down(&mut self) {
        let max_row = self.used_range.rows.saturating_sub(1);
        if self.cursor.row < max_row {
            self.cursor.row += 1;
        } else {
            self.used_range.rows = self.used_range.rows.saturating_add(1);
            self.cursor.row += 1;
        }
        self.ensure_cursor_visible();
    }

    pub fn move_left(&mut self) {
        if self.cursor.col > 0 {
            self.cursor.col -= 1;
        }
        self.ensure_cursor_visible();
    }

    pub fn move_right(&mut self) {
        let max_col = self.used_range.cols.saturating_sub(1);
        if self.cursor.col < max_col {
            self.cursor.col += 1;
        } else {
            self.used_range.cols = self.used_range.cols.saturating_add(1);
            self.cursor.col += 1;
        }
        self.ensure_cursor_visible();
    }

    pub fn start_edit(&mut self) {
        self.mode = SheetMode::Edit;
        self.edit_buffer = self
            .get_cell(self.cursor.row, self.cursor.col)
            .unwrap_or_default();
    }

    pub fn cancel_edit(&mut self) {
        self.mode = SheetMode::View;
        self.edit_buffer.clear();
    }

    pub fn set_cell(&mut self, row: usize, col: usize, value: String) {
        if row >= self.values.len() {
            self.values.resize_with(row + 1, Vec::new);
        }
        if col >= self.values[row].len() {
            self.values[row].resize(col + 1, String::new());
        }
        self.values[row][col] = value;
        self.used_range.rows = self.values.len().max(1);
        self.used_range.cols = self
            .values
            .iter()
            .map(|r| r.len())
            .max()
            .unwrap_or(1)
            .max(1);
    }

    pub fn get_cell(&self, row: usize, col: usize) -> Option<String> {
        self.values
            .get(row)
            .and_then(|r| r.get(col))
            .cloned()
    }

    pub fn switch_tab(&mut self, delta: isize) {
        if self.tabs.is_empty() {
            return;
        }
        let len = self.tabs.len() as isize;
        let mut next = self.active_tab as isize + delta;
        if next < 0 {
            next = len - 1;
        } else if next >= len {
            next = 0;
        }
        self.active_tab = next as usize;
        self.cursor = SheetCursor::new();
        self.viewport = SheetViewport::new();
        self.values.clear();
        self.used_range = UsedRange::new();
    }

    fn ensure_cursor_in_bounds(&mut self) {
        if self.cursor.row >= self.used_range.rows {
            self.cursor.row = self.used_range.rows.saturating_sub(1);
        }
        if self.cursor.col >= self.used_range.cols {
            self.cursor.col = self.used_range.cols.saturating_sub(1);
        }
    }

    fn ensure_cursor_visible(&mut self) {
        if self.viewport.height == 0 || self.viewport.width == 0 {
            return;
        }

        if self.cursor.row < self.viewport.row {
            self.viewport.row = self.cursor.row;
        } else if self.cursor.row >= self.viewport.row + self.viewport.height {
            self.viewport.row = self.cursor.row + 1 - self.viewport.height;
        }

        if self.cursor.col < self.viewport.col {
            self.viewport.col = self.cursor.col;
        } else if self.cursor.col >= self.viewport.col + self.viewport.width {
            self.viewport.col = self.cursor.col + 1 - self.viewport.width;
        }
    }
}

impl Default for SheetGridState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct SheetsState {
    pub view_mode: SheetsViewMode,
    pub browser: SheetsBrowserState,
    pub grid: SheetGridState,
    pub last_sync: Option<DateTime<Utc>>,
}

impl SheetsState {
    pub fn new() -> Self {
        Self {
            view_mode: SheetsViewMode::Browser,
            browser: SheetsBrowserState::new(),
            grid: SheetGridState::new(),
            last_sync: None,
        }
    }

    pub fn open_browser(&mut self) {
        self.view_mode = SheetsViewMode::Browser;
        self.browser.reset();
    }

    pub fn open_grid(&mut self) {
        self.view_mode = SheetsViewMode::Grid;
    }
}

impl Default for SheetsState {
    fn default() -> Self {
        Self::new()
    }
}
