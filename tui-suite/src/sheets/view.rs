use chrono::Local;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Table, Row, Cell};
use ratatui::Frame;

use super::state::{SheetMode, SheetsState, SheetsViewMode};

pub fn render_sheets(f: &mut Frame, area: Rect, state: &mut SheetsState) {
    match state.view_mode {
        SheetsViewMode::Browser => render_browser(f, area, state),
        SheetsViewMode::Grid => render_grid(f, area, state),
    }
}

fn render_browser(f: &mut Frame, area: Rect, state: &mut SheetsState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(area);

    let search_title = "Spreadsheet Search (type to filter)";
    let search = Paragraph::new(state.browser.query.clone())
        .block(Block::default().title(search_title).borders(Borders::ALL));
    f.render_widget(search, chunks[0]);

    f.set_cursor_position((
        chunks[0].x + 1 + state.browser.query.len() as u16,
        chunks[0].y + 1,
    ));

    let items: Vec<ListItem> = if let Some(ref error) = state.browser.error {
        vec![ListItem::new(error.clone())]
    } else if state.browser.loading && state.browser.docs.is_empty() {
        vec![ListItem::new("Loading...")]
    } else if state.browser.docs.is_empty() {
        vec![ListItem::new("No spreadsheets found")]
    } else {
        state
            .browser
            .docs
            .iter()
            .map(|doc| {
                let date = doc
                    .modified_time
                    .map(|dt| dt.with_timezone(&Local).format("%Y-%m-%d").to_string())
                    .unwrap_or_else(|| "-".to_string());
                let line = format!("{:12}  {}", date, doc.name);
                ListItem::new(line)
            })
            .collect()
    };

    let list = List::new(items)
        .block(
            Block::default()
                .title("Spreadsheets (Enter: open)")
                .borders(Borders::ALL),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");
    f.render_stateful_widget(list, chunks[1], &mut state.browser.list_state);
}

fn render_grid(f: &mut Frame, area: Rect, state: &mut SheetsState) {
    let mut constraints = Vec::new();
    constraints.push(Constraint::Length(1));
    constraints.push(Constraint::Min(1));
    if state.grid.mode == SheetMode::Edit {
        constraints.push(Constraint::Length(3));
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    render_tabs(f, chunks[0], state);
    render_table(f, chunks[1], state);

    if state.grid.mode == SheetMode::Edit {
        render_edit_bar(f, chunks[2], state);
    }
}

fn render_tabs(f: &mut Frame, area: Rect, state: &SheetsState) {
    if state.grid.tabs.is_empty() {
        let title = "No sheets loaded";
        f.render_widget(Paragraph::new(title), area);
        return;
    }

    let mut spans = Vec::new();
    for (i, tab) in state.grid.tabs.iter().enumerate() {
        let style = if i == state.grid.active_tab {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        spans.push(Span::styled(format!(" {} ", tab.name), style));
        if i + 1 < state.grid.tabs.len() {
            spans.push(Span::raw("│"));
        }
    }

    let bar = Paragraph::new(Line::from(spans));
    f.render_widget(bar, area);
}

fn render_table(f: &mut Frame, area: Rect, state: &mut SheetsState) {
    let grid_area = area;
    if grid_area.height < 2 || grid_area.width < 10 {
        let msg = Paragraph::new("Terminal too small");
        f.render_widget(msg, grid_area);
        return;
    }

    let header_height = 1usize;
    let visible_rows = grid_area.height.saturating_sub(header_height as u16) as usize;
    let available_width = grid_area.width as usize;

    let row_header_width = (state.grid.used_range.rows.max(1)).to_string().len().max(2) + 1;

    let start_row = state.grid.viewport.row;
    let start_col = state.grid.viewport.col;

    let (col_widths, col_labels) = compute_visible_columns(
        state,
        available_width,
        row_header_width,
        start_row,
        visible_rows,
        start_col,
    );

    state
        .grid
        .set_viewport_size(visible_rows, col_widths.len());

    let mut header_cells = Vec::new();
    header_cells.push(Cell::from(" "));
    for label in col_labels.iter() {
        header_cells.push(Cell::from(label.clone()));
    }
    let header = Row::new(header_cells)
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

    let mut rows = Vec::new();
    for row_idx in 0..visible_rows {
        let sheet_row = start_row + row_idx;
        let mut cells = Vec::new();
        cells.push(Cell::from(format!("{}", sheet_row + 1)));

        for (col_offset, _) in col_widths.iter().enumerate() {
            let sheet_col = start_col + col_offset;
            let value = state
                .grid
                .get_cell(sheet_row, sheet_col)
                .unwrap_or_default();
            let mut cell = Cell::from(value);
            if sheet_row == state.grid.cursor.row && sheet_col == state.grid.cursor.col {
                cell = cell.style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                );
            }
            cells.push(cell);
        }
        rows.push(Row::new(cells));
    }

    let mut constraints = Vec::new();
    constraints.push(Constraint::Length(row_header_width as u16));
    for width in col_widths.iter() {
        constraints.push(Constraint::Length(*width as u16));
    }

    let table = Table::new(rows, constraints)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Sheet"))
        .column_spacing(1);

    f.render_widget(table, grid_area);
}

fn render_edit_bar(f: &mut Frame, area: Rect, state: &SheetsState) {
    let input = Paragraph::new(state.grid.edit_buffer.clone())
        .block(Block::default().title("Edit Cell (Enter: save, Esc: cancel)").borders(Borders::ALL));
    f.render_widget(input, area);
    f.set_cursor_position((
        area.x + 1 + state.grid.edit_buffer.len() as u16,
        area.y + 1,
    ));
}

fn compute_visible_columns(
    state: &SheetsState,
    available_width: usize,
    row_header_width: usize,
    start_row: usize,
    visible_rows: usize,
    start_col: usize,
) -> (Vec<usize>, Vec<String>) {
    let mut widths = Vec::new();
    let mut labels = Vec::new();
    let mut total = row_header_width + 2;
    let max_width = 20usize;
    let min_width = 6usize;

    let total_cols = state.grid.used_range.cols.max(1);
    for col in start_col..total_cols {
        let label = col_to_label(col);
        let mut width = label.len();

        for row in start_row..(start_row + visible_rows) {
            if let Some(value) = state.grid.get_cell(row, col) {
                width = width.max(value.len());
            }
        }

        width = width.clamp(min_width, max_width);
        if total + width + 1 > available_width {
            break;
        }
        widths.push(width);
        labels.push(label);
        total += width + 1;
    }

    if widths.is_empty() {
        widths.push(min_width);
        labels.push(col_to_label(start_col));
    }

    (widths, labels)
}

fn col_to_label(mut col: usize) -> String {
    let mut label = String::new();
    loop {
        let rem = col % 26;
        label.insert(0, (b'A' + rem as u8) as char);
        if col < 26 {
            break;
        }
        col = (col / 26) - 1;
    }
    label
}
