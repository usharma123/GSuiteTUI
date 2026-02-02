use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};
use ratatui::Frame;

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
        self.ensure_cursor_visible(height);

        // Render line numbers
        self.render_line_numbers(f, line_num_area, height, line_num_width);

        // Render separator
        let separator_lines: Vec<Line> = (0..height)
            .map(|_| Line::from(Span::styled("│", Style::default().fg(Color::DarkGray))))
            .collect();
        f.render_widget(Paragraph::new(separator_lines), separator_area);

        // Render visible lines with markdown syntax highlighting
        let mut text = Text::default();
        for i in 0..height {
            let line_idx = self.top_line + i;
            if line_idx >= self.doc.len_lines() {
                // Empty line for visual consistency
                text.lines.push(Line::from(Span::raw("")));
            } else {
                let mut s = self.doc.line(line_idx).to_string();
                s = s.trim_end_matches('\n').to_string();

                // Highlight current line with subtle background
                let mut styled_line = style_markdown_line(&s);
                if line_idx == self.doc.cursor.line {
                    // Add subtle highlight to current line
                    styled_line = styled_line.style(Style::default().bg(Color::Rgb(40, 44, 52)));
                }
                text.lines.push(styled_line);
            }
        }

        let paragraph = Paragraph::new(text);
        f.render_widget(paragraph, content_area);

        // Render scrollbar if needed
        if total_lines > height {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("▲"))
                .end_symbol(Some("▼"))
                .track_symbol(Some("│"))
                .thumb_symbol("█");

            let mut scrollbar_state = ScrollbarState::new(total_lines)
                .position(self.top_line)
                .viewport_content_length(height);

            f.render_stateful_widget(
                scrollbar,
                editor_area.inner(Margin { vertical: 1, horizontal: 0 }),
                &mut scrollbar_state,
            );
        }

        // Cursor position (adjusted for line numbers)
        let cursor_y = (self.doc.cursor.line.saturating_sub(self.top_line)) as u16;
        let cursor_x = self.doc.cursor.col as u16;
        if cursor_y < content_area.height && cursor_x < content_area.width {
            f.set_cursor_position((content_area.x + cursor_x, content_area.y + cursor_y));
        }
    }

    fn render_line_numbers(&self, f: &mut Frame, area: Rect, height: usize, width: usize) {
        let mut lines = Vec::new();
        for i in 0..height {
            let line_idx = self.top_line + i;
            if line_idx >= self.doc.len_lines() {
                lines.push(Line::from(Span::styled(
                    format!("{:>width$}", "~", width = width - 1),
                    Style::default().fg(Color::DarkGray),
                )));
            } else {
                let line_num = line_idx + 1;
                let style = if line_idx == self.doc.cursor.line {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                lines.push(Line::from(Span::styled(
                    format!("{:>width$}", line_num, width = width - 1),
                    style,
                )));
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
}
