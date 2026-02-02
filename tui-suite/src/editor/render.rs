use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};
use ratatui::Frame;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::markdown::style_markdown_line;
use super::ops::EditorState;

impl EditorState {
    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        // Split area: main editor + help bar at bottom
        let chunks = Layout::vertical([
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .split(area);

        let editor_area = chunks[0];
        let help_area = chunks[1];

        // Render the help bar
        self.render_help_bar(f, help_area);

        // Create styled block with rounded borders
        let total_lines = self.doc.len_lines();
        let cursor_line = self.doc.cursor.line + 1;
        let cursor_col = self.doc.cursor.col + 1;

        // Build title with modified indicator
        let source_title = self.source_title();
        let source_label = if self.drive_doc().is_some() {
            format!("(Drive) {}", source_title)
        } else {
            source_title
        };
        let mut title_spans = vec![
            Span::styled(" Editor ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("(Markdown) ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{} ", source_label), Style::default().fg(Color::DarkGray)),
        ];
        
        // Add save status indicator
        if self.modified {
            title_spans.push(Span::styled(
                "[Modified] ",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ));
        } else {
            title_spans.push(Span::styled(
                "[Saved] ",
                Style::default().fg(Color::Green),
            ));
        }

        let block = Block::default()
            .title(Line::from(title_spans))
            .title_bottom(Line::from(vec![
                Span::styled(" Ln ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{}", cursor_line), Style::default().fg(Color::Yellow)),
                Span::styled(", Col ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{}", cursor_col), Style::default().fg(Color::Yellow)),
                Span::styled(format!(" | {} lines ", total_lines), Style::default().fg(Color::DarkGray)),
            ]).right_aligned())
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(Color::Blue));

        let inner = block.inner(editor_area);
        f.render_widget(block, editor_area);

        // Calculate line number width
        let line_num_width = total_lines.to_string().len().max(3) + 1;

        // Split inner area for line numbers and content
        let content_chunks = Layout::horizontal([
            Constraint::Length(line_num_width as u16),
            Constraint::Length(1), // separator
            Constraint::Min(1),    // content
        ])
        .split(inner);

        let line_num_area = content_chunks[0];
        let separator_area = content_chunks[1];
        let content_area = content_chunks[2];

        let height = content_area.height as usize;
        let wrap_width = content_area.width.max(1) as usize;
        self.ensure_cursor_visible(height, wrap_width);

        let top_visual_row = self.visual_index_for_line(self.top_line, wrap_width)
            + self.top_line_offset;
        let total_visual_rows = self.total_visual_rows(wrap_width);

        // Render line numbers (aligned with visual lines)
        let (visual_lines, line_numbers) =
            self.build_visual_lines(height, wrap_width);
        self.render_line_numbers(f, line_num_area, &line_numbers, line_num_width);

        // Render separator
        let separator_lines: Vec<Line> = (0..height)
            .map(|_| Line::from(Span::styled("│", Style::default().fg(Color::DarkGray))))
            .collect();
        f.render_widget(Paragraph::new(separator_lines), separator_area);

        // Render visible lines with markdown syntax highlighting and wrapping
        let mut text = Text::default();
        for line in visual_lines {
            text.lines.push(line);
        }

        let paragraph = Paragraph::new(text).wrap(ratatui::widgets::Wrap { trim: false });
        f.render_widget(paragraph, content_area);

        // Render scrollbar if needed
        if total_visual_rows > height {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("▲"))
                .end_symbol(Some("▼"))
                .track_symbol(Some("│"))
                .thumb_symbol("█");

            let mut scrollbar_state = ScrollbarState::new(total_visual_rows)
                .position(top_visual_row)
                .viewport_content_length(height);

            f.render_stateful_widget(
                scrollbar,
                editor_area.inner(Margin { vertical: 1, horizontal: 0 }),
                &mut scrollbar_state,
            );
        }

        // Cursor position (adjusted for wrapped lines)
        if let Some((cursor_y, cursor_x)) =
            self.cursor_visual_position(wrap_width, top_visual_row)
        {
            if cursor_y < content_area.height && cursor_x < content_area.width {
                f.set_cursor_position((content_area.x + cursor_x, content_area.y + cursor_y));
            }
        }
    }

    fn render_line_numbers(&self, f: &mut Frame, area: Rect, numbers: &[Option<usize>], width: usize) {
        let mut lines = Vec::new();
        for number in numbers {
            match number {
                Some(line_num) => {
                    let style = if *line_num == self.doc.cursor.line + 1 {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };
                    lines.push(Line::from(Span::styled(
                        format!("{:>width$}", line_num, width = width - 1),
                        style,
                    )));
                }
                None => {
                    lines.push(Line::from(Span::styled(
                        format!("{:>width$}", " ", width = width - 1),
                        Style::default().fg(Color::DarkGray),
                    )));
                }
            }
        }
        f.render_widget(Paragraph::new(lines), area);
    }

    fn render_help_bar(&self, f: &mut Frame, area: Rect) {
        let help_spans = vec![
            Span::styled(" ^S", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" Save ", Style::default().fg(Color::DarkGray)),
            Span::styled("^B", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" Bold ", Style::default().fg(Color::DarkGray)),
            Span::styled("^I", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" Italic ", Style::default().fg(Color::DarkGray)),
            Span::styled("Alt+1/2/3", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" H1/H2/H3 ", Style::default().fg(Color::DarkGray)),
            Span::styled("^Z", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" Undo ", Style::default().fg(Color::DarkGray)),
            Span::styled("^Y", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" Redo ", Style::default().fg(Color::DarkGray)),
            Span::styled("Tab", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" Switch ", Style::default().fg(Color::DarkGray)),
        ];

        let help_line = Line::from(help_spans);
        let help_paragraph = Paragraph::new(help_line)
            .style(Style::default().bg(Color::Rgb(30, 30, 40)));
        f.render_widget(help_paragraph, area);
    }

    fn build_visual_lines(&self, height: usize, wrap_width: usize) -> (Vec<Line<'static>>, Vec<Option<usize>>) {
        let mut lines = Vec::new();
        let mut numbers = Vec::new();

        let mut line_idx = self.top_line;
        let mut skip_in_line = self.top_line_offset;

        while lines.len() < height {
            if line_idx >= self.doc.len_lines() {
                lines.push(Line::from(Span::raw("")));
                numbers.push(None);
                continue;
            }

            let mut s = self.doc.line(line_idx).to_string();
            s = s.trim_end_matches('\n').to_string();

            let mut styled_line = style_markdown_line(&s);
            if line_idx == self.doc.cursor.line {
                styled_line = styled_line.style(Style::default().bg(Color::Rgb(40, 44, 52)));
            }

            let wrapped = wrap_styled_line(styled_line, wrap_width);
            let start = skip_in_line.min(wrapped.len());
            for (i, line) in wrapped.into_iter().enumerate().skip(start) {
                lines.push(line);
                if i == 0 {
                    numbers.push(Some(line_idx + 1));
                } else {
                    numbers.push(None);
                }
                if lines.len() >= height {
                    break;
                }
            }

            skip_in_line = 0;
            line_idx += 1;
        }

        (lines, numbers)
    }

    fn total_visual_rows(&self, wrap_width: usize) -> usize {
        let mut total = 0;
        for i in 0..self.doc.len_lines() {
            let mut s = self.doc.line(i).to_string();
            s = s.trim_end_matches('\n').to_string();
            total += visual_rows_for_text(&s, wrap_width);
        }
        total.max(1)
    }

    fn visual_index_for_line(&self, line_idx: usize, wrap_width: usize) -> usize {
        let mut total = 0;
        for i in 0..line_idx.min(self.doc.len_lines()) {
            let mut s = self.doc.line(i).to_string();
            s = s.trim_end_matches('\n').to_string();
            total += visual_rows_for_text(&s, wrap_width);
        }
        total
    }

    fn cursor_visual_position(&self, wrap_width: usize, top_visual_row: usize) -> Option<(u16, u16)> {
        if wrap_width == 0 {
            return None;
        }
        let cursor_line = self.doc.cursor.line;
        if cursor_line >= self.doc.len_lines() {
            return None;
        }

        let mut s = self.doc.line(cursor_line).to_string();
        s = s.trim_end_matches('\n').to_string();
        let col_width = visual_width_for_col(&s, self.doc.cursor.col);
        let row_in_line = col_width / wrap_width;
        let col_in_row = col_width % wrap_width;
        let cursor_visual = self.visual_index_for_line(cursor_line, wrap_width) + row_in_line;
        if cursor_visual < top_visual_row {
            return None;
        }
        let cursor_y = cursor_visual - top_visual_row;
        Some((cursor_y as u16, col_in_row as u16))
    }

    fn ensure_cursor_visible(&mut self, height: usize, wrap_width: usize) {
        if height == 0 {
            return;
        }
        let cursor_visual = self.cursor_visual_index(wrap_width);
        let top_visual = self.visual_index_for_line(self.top_line, wrap_width) + self.top_line_offset;
        let bottom_visual = top_visual + height.saturating_sub(1);

        if cursor_visual < top_visual {
            let (line, offset) = self.visual_index_to_line_offset(cursor_visual, wrap_width);
            self.top_line = line;
            self.top_line_offset = offset;
        } else if cursor_visual > bottom_visual {
            let target = cursor_visual.saturating_sub(height.saturating_sub(1));
            let (line, offset) = self.visual_index_to_line_offset(target, wrap_width);
            self.top_line = line;
            self.top_line_offset = offset;
        }
    }

    fn cursor_visual_index(&self, wrap_width: usize) -> usize {
        let cursor_line = self.doc.cursor.line;
        let mut total = self.visual_index_for_line(cursor_line, wrap_width);
        if cursor_line >= self.doc.len_lines() {
            return total;
        }
        let mut s = self.doc.line(cursor_line).to_string();
        s = s.trim_end_matches('\n').to_string();
        let col_width = visual_width_for_col(&s, self.doc.cursor.col);
        total += col_width / wrap_width.max(1);
        total
    }

    fn visual_index_to_line_offset(&self, visual_index: usize, wrap_width: usize) -> (usize, usize) {
        let mut remaining = visual_index;
        let total_lines = self.doc.len_lines();
        for i in 0..total_lines {
            let mut s = self.doc.line(i).to_string();
            s = s.trim_end_matches('\n').to_string();
            let rows = visual_rows_for_text(&s, wrap_width);
            if remaining < rows {
                return (i, remaining);
            }
            remaining -= rows;
        }
        (total_lines.saturating_sub(1), 0)
    }
}

fn visual_rows_for_text(text: &str, wrap_width: usize) -> usize {
    if wrap_width == 0 {
        return 1;
    }
    let width = UnicodeWidthStr::width(text);
    let rows = (width + wrap_width - 1) / wrap_width;
    rows.max(1)
}

fn visual_width_for_col(text: &str, col: usize) -> usize {
    let mut width = 0;
    for (i, ch) in text.chars().enumerate() {
        if i >= col {
            break;
        }
        width += UnicodeWidthChar::width(ch).unwrap_or(0);
    }
    width
}

fn wrap_styled_line(line: Line<'static>, wrap_width: usize) -> Vec<Line<'static>> {
    if wrap_width == 0 {
        return vec![line];
    }
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut current_spans: Vec<Span<'static>> = Vec::new();
    let mut current_width = 0usize;

    for span in line.spans {
        let style = span.style;
        let text = span.content.to_string();
        if text.is_empty() {
            continue;
        }
        let mut chunk = String::new();
        for ch in text.chars() {
            let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
            if current_width + ch_width > wrap_width && current_width > 0 {
                if !chunk.is_empty() {
                    current_spans.push(Span::styled(chunk.clone(), style));
                    chunk.clear();
                }
                lines.push(Line::from(current_spans));
                current_spans = Vec::new();
                current_width = 0;
            }

            chunk.push(ch);
            current_width += ch_width;

            if current_width >= wrap_width {
                if !chunk.is_empty() {
                    current_spans.push(Span::styled(chunk.clone(), style));
                    chunk.clear();
                }
                lines.push(Line::from(current_spans));
                current_spans = Vec::new();
                current_width = 0;
            }
        }
        if !chunk.is_empty() {
            current_spans.push(Span::styled(chunk, style));
        }
    }

    if current_spans.is_empty() {
        lines.push(Line::from(Span::raw("")));
    } else {
        lines.push(Line::from(current_spans));
    }

    lines
}
