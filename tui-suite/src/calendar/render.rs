use chrono::{Datelike, Duration, Local, NaiveDate, Weekday, Timelike};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use super::layout::compute_layout;
use super::model::{CalendarState, EventStatus};

impl CalendarState {
    pub fn render_week(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(format!(
                "Calendar - Week of {}",
                self.week_start.format("%b %d, %Y")
            ))
            .borders(Borders::ALL);
        let inner = block.inner(area);
        f.render_widget(block, area);

        // Layout: time gutter + 7 day columns
        let mut constraints = vec![Constraint::Length(6)]; // Time gutter
        for _ in 0..7 {
            constraints.push(Constraint::Ratio(1, 7));
        }

        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(inner);

        // Render time gutter
        self.render_time_gutter(f, columns[0]);

        // Render each day column
        for day in 0..7 {
            let date = self.week_start + Duration::days(day as i64);
            self.render_day_column(f, columns[day + 1], date, day);
        }
    }

    fn render_time_gutter(&self, f: &mut Frame, area: Rect) {
        let visible_rows = area.height as usize;
        let mut lines = Vec::new();

        for row in 0..visible_rows {
            let time_row = self.scroll_offset as usize + row;
            if time_row >= 48 {
                break;
            }

            // Show hour labels on even rows
            if time_row % 2 == 0 {
                let hour = time_row / 2;
                let label = if hour == 0 {
                    "12 AM".to_string()
                } else if hour < 12 {
                    format!("{:2} AM", hour)
                } else if hour == 12 {
                    "12 PM".to_string()
                } else {
                    format!("{:2} PM", hour - 12)
                };
                lines.push(Line::from(label));
            } else {
                lines.push(Line::from("     "));
            }
        }

        let text = Text::from(lines);
        let paragraph = Paragraph::new(text);
        f.render_widget(paragraph, area);
    }

    fn render_day_column(&self, f: &mut Frame, area: Rect, date: NaiveDate, _day_idx: usize) {
        let today = Local::now().date_naive();
        let is_today = date == today;

        // Header with day name
        let header_style = if is_today {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().add_modifier(Modifier::BOLD)
        };

        let day_name = match date.weekday() {
            Weekday::Mon => "Mon",
            Weekday::Tue => "Tue",
            Weekday::Wed => "Wed",
            Weekday::Thu => "Thu",
            Weekday::Fri => "Fri",
            Weekday::Sat => "Sat",
            Weekday::Sun => "Sun",
        };

        let header = format!("{} {}", day_name, date.day());
        let header_para = Paragraph::new(header)
            .style(header_style)
            .alignment(Alignment::Center);

        // Split area into header and body
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(1)])
            .split(area);

        f.render_widget(header_para, chunks[0]);

        // Render events
        self.render_day_events(f, chunks[1], date);
    }

    fn render_day_events(&self, f: &mut Frame, area: Rect, date: NaiveDate) {
        let layout_events = compute_layout(&self.events, date);
        let visible_rows = area.height as usize;
        let scroll = self.scroll_offset as usize;

        // Draw time grid lines (faint)
        for row in 0..visible_rows {
            let time_row = scroll + row;
            if time_row >= 48 {
                break;
            }

            // Hour lines
            if time_row % 2 == 0 {
                let line = "─".repeat(area.width as usize);
                let para = Paragraph::new(line).style(Style::default().fg(Color::DarkGray));
                let line_area = Rect::new(area.x, area.y + row as u16, area.width, 1);
                f.render_widget(para, line_area);
            }
        }

        // Render events
        for layout_event in &layout_events {
            let start_row = layout_event.start_row;
            let end_row = layout_event.end_row;

            // Check if visible
            if end_row <= scroll || start_row >= scroll + visible_rows {
                continue;
            }

            // Calculate visible portion
            let vis_start = start_row.saturating_sub(scroll);
            let vis_end = (end_row - scroll).min(visible_rows);

            if vis_start >= vis_end {
                continue;
            }

            // Calculate horizontal position based on lane
            let lane_width = area.width / layout_event.total_lanes as u16;
            let x = area.x + (layout_event.lane as u16 * lane_width);
            let width = lane_width.saturating_sub(1).max(1);

            let event_area = Rect::new(
                x,
                area.y + vis_start as u16,
                width,
                (vis_end - vis_start) as u16,
            );

            // Style based on status
            let style = match layout_event.event.status {
                EventStatus::Confirmed => Style::default().bg(Color::Blue).fg(Color::White),
                EventStatus::Tentative => Style::default().bg(Color::Yellow).fg(Color::Black),
                EventStatus::Cancelled => Style::default()
                    .bg(Color::Red)
                    .fg(Color::White)
                    .add_modifier(Modifier::CROSSED_OUT),
            };

            // Truncate summary to fit
            let summary = &layout_event.event.summary;
            let display_text = if summary.len() > width as usize {
                format!("{}...", &summary[..width.saturating_sub(3) as usize])
            } else {
                summary.clone()
            };

            let block = Block::default().style(style);
            let inner = block.inner(event_area);
            f.render_widget(block, event_area);

            if inner.height > 0 {
                let text = Paragraph::new(display_text).style(style);
                f.render_widget(text, inner);
            }
        }

        // Draw current time line if today
        let now = Local::now();
        if now.date_naive() == date {
            let now_row = (now.hour() * 2 + now.minute() / 30) as usize;
            if now_row >= scroll && now_row < scroll + visible_rows {
                let y = area.y + (now_row - scroll) as u16;
                let line = "━".repeat(area.width as usize);
                let para = Paragraph::new(line).style(Style::default().fg(Color::Red));
                let line_area = Rect::new(area.x, y, area.width, 1);
                f.render_widget(para, line_area);
            }
        }
    }
}
