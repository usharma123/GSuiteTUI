use chrono::{Datelike, Duration, Local, NaiveDate, Timelike, Weekday};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use unicode_width::UnicodeWidthStr;

use super::layout::compute_layout;
use super::model::{CalendarEvent, CalendarState, EventStatus};

/// Truncate text to fit within max_width, accounting for unicode character widths
fn clip_text(s: &str, max_width: usize) -> String {
    let width = UnicodeWidthStr::width(s);
    if width <= max_width {
        return s.to_string();
    }
    if max_width <= 1 {
        return "…".to_string();
    }

    let mut out = String::new();
    let mut current_width = 0;
    let target_width = max_width.saturating_sub(1); // Leave room for ellipsis

    for ch in s.chars() {
        let ch_width = UnicodeWidthStr::width(ch.to_string().as_str());
        if current_width + ch_width > target_width {
            break;
        }
        out.push(ch);
        current_width += ch_width;
    }
    out.push('…');
    out
}

/// Draw a horizontal line directly to the buffer
fn draw_hline(buf: &mut Buffer, y: u16, x0: u16, x1: u16, ch: &str, style: Style) {
    for x in x0..x1 {
        if let Some(cell) = buf.cell_mut((x, y)) {
            cell.set_symbol(ch).set_style(style);
        }
    }
}

/// Fill a rect with a background color
fn fill_bg(buf: &mut Buffer, rect: Rect, style: Style) {
    for y in rect.y..rect.y + rect.height {
        for x in rect.x..rect.x + rect.width {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_style(style);
            }
        }
    }
}

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
        let today = Local::now().date_naive();
        for day in 0..7 {
            let date = self.week_start + Duration::days(day as i64);
            let is_today = date == today;
            let is_selected = day == self.selected_day;
            self.render_day_column(f, columns[day + 1], date, day, is_today, is_selected);
        }
    }

    fn render_time_gutter(&self, f: &mut Frame, area: Rect) {
        let visible_rows = area.height as usize;
        let scroll = self.scroll_offset as usize;
        let buf = f.buffer_mut();

        for row in 0..visible_rows {
            let time_row = scroll + row;
            if time_row >= 48 {
                break;
            }

            let y = area.y + row as u16;

            // Show hour labels on even rows (hour boundaries)
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

                // Render label
                for (i, ch) in label.chars().enumerate() {
                    if i < area.width as usize {
                        if let Some(cell) = buf.cell_mut((area.x + i as u16, y)) {
                            cell.set_char(ch)
                                .set_style(Style::default().fg(Color::Gray));
                        }
                    }
                }
            }
        }

        // Draw now marker in gutter if current week
        let now = Local::now();
        let today = now.date_naive();
        if self.week_start <= today && today <= self.week_end() {
            let now_row = (now.hour() * 2 + now.minute() / 30) as usize;
            if now_row >= scroll && now_row < scroll + visible_rows {
                let y = area.y + (now_row - scroll) as u16;
                if let Some(cell) = buf.cell_mut((area.x + area.width - 1, y)) {
                    cell.set_symbol("▶")
                        .set_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
                }
            }
        }
    }

    fn render_day_column(
        &self,
        f: &mut Frame,
        area: Rect,
        date: NaiveDate,
        day_idx: usize,
        is_today: bool,
        is_selected: bool,
    ) {
        let buf = f.buffer_mut();

        // Draw vertical separator on left edge (except first column)
        if day_idx > 0 {
            for y in area.y..area.y + area.height {
                if let Some(cell) = buf.cell_mut((area.x, y)) {
                    cell.set_symbol("│")
                        .set_style(Style::default().fg(Color::DarkGray));
                }
            }
        }

        // Adjust area to account for separator
        let content_area = if day_idx > 0 {
            Rect::new(area.x + 1, area.y, area.width.saturating_sub(1), area.height)
        } else {
            area
        };

        // Fill background for selected/today column
        if is_selected || is_today {
            let bg_color = if is_selected {
                Color::Rgb(25, 30, 40) // Subtle blue tint for selected
            } else {
                Color::Rgb(20, 25, 30) // Subtle tint for today
            };
            fill_bg(buf, content_area, Style::default().bg(bg_color));
        }

        // Header style
        let header_style = if is_today {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else if is_selected {
            Style::default()
                .fg(Color::Cyan)
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

        // Get all-day events for this day
        let all_day_events = self.all_day_events_for_day(date);
        let all_day_height = if all_day_events.is_empty() {
            0
        } else {
            (all_day_events.len() as u16).min(2) + 1
        };

        // Split: header (1) + all-day strip (0-3) + time grid
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(all_day_height),
                Constraint::Min(1),
            ])
            .split(content_area);

        // Render header
        let header_para = Paragraph::new(header)
            .style(header_style)
            .alignment(Alignment::Center);
        f.render_widget(header_para, chunks[0]);

        // Render all-day strip if there are all-day events
        if all_day_height > 0 {
            self.render_all_day_strip(f, chunks[1], &all_day_events);
        }

        // Render timed events
        self.render_day_events(f, chunks[2], date, is_today);
    }

    fn render_all_day_strip(&self, f: &mut Frame, area: Rect, events: &[&CalendarEvent]) {
        let buf = f.buffer_mut();

        // Subtle background for all-day strip
        fill_bg(buf, area, Style::default().bg(Color::Rgb(35, 40, 50)));

        // Render events as pills
        for (i, event) in events.iter().take(area.height as usize).enumerate() {
            let y = area.y + i as u16;
            let text = clip_text(&event.summary, area.width.saturating_sub(2) as usize);

            let style = match event.status {
                EventStatus::Confirmed => Style::default().bg(Color::Cyan).fg(Color::Black),
                EventStatus::Tentative => Style::default().bg(Color::Yellow).fg(Color::Black),
                EventStatus::Cancelled => Style::default()
                    .bg(Color::Red)
                    .fg(Color::White)
                    .add_modifier(Modifier::CROSSED_OUT),
            };

            // Draw pill background
            for x in area.x..area.x + area.width {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_style(style);
                }
            }

            // Draw text centered
            let text_start = area.x + 1;
            for (i, ch) in text.chars().enumerate() {
                if text_start + (i as u16) < area.x + area.width - 1 {
                    if let Some(cell) = buf.cell_mut((text_start + i as u16, y)) {
                        cell.set_char(ch);
                    }
                }
            }
        }
    }

    fn render_day_events(&self, f: &mut Frame, area: Rect, date: NaiveDate, is_today: bool) {
        let layout_events = compute_layout(&self.events, date);
        let visible_rows = area.height as usize;
        let scroll = self.scroll_offset as usize;
        let buf = f.buffer_mut();

        // Draw time grid lines
        for row in 0..visible_rows {
            let time_row = scroll + row;
            if time_row >= 48 {
                break;
            }

            let y = area.y + row as u16;

            if time_row % 2 == 0 {
                // Hour lines - stronger
                draw_hline(buf, y, area.x, area.x + area.width, "─",
                    Style::default().fg(Color::Gray));
            } else {
                // Half-hour lines - subtle/dotted
                draw_hline(buf, y, area.x, area.x + area.width, "┄",
                    Style::default().fg(Color::DarkGray));
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

            // Calculate horizontal position based on lane with gaps
            let total_lanes = layout_event.total_lanes as u16;
            let gap = if total_lanes > 1 { 1u16 } else { 0u16 };
            let total_gap = gap * (total_lanes.saturating_sub(1));
            let available = area.width.saturating_sub(total_gap);
            let lane_width = (available / total_lanes).max(1);

            let x = area.x + (layout_event.lane as u16 * (lane_width + gap));
            let width = lane_width.saturating_sub(1).max(1);

            let event_rect = Rect::new(
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

            // Fill event background
            fill_bg(buf, event_rect, style);

            // Build first line: HH:MM Title
            if event_rect.height > 0 && event_rect.width > 2 {
                let start_time = layout_event
                    .event
                    .start
                    .time()
                    .map(|t| format!("{:02}:{:02} ", t.hour(), t.minute()))
                    .unwrap_or_default();

                let available_width = (event_rect.width as usize).saturating_sub(2);
                let time_width = UnicodeWidthStr::width(start_time.as_str());
                let title_width = available_width.saturating_sub(time_width);

                let display_text = format!(
                    "{}{}",
                    start_time,
                    clip_text(&layout_event.event.summary, title_width)
                );

                // Render text
                let text_y = event_rect.y;
                let text_x = event_rect.x + 1;
                for (i, ch) in display_text.chars().enumerate() {
                    if text_x + (i as u16) < event_rect.x + event_rect.width - 1 {
                        if let Some(cell) = buf.cell_mut((text_x + i as u16, text_y)) {
                            cell.set_char(ch);
                        }
                    }
                }
            }

            // Draw left border of event block
            for y in event_rect.y..event_rect.y + event_rect.height {
                if let Some(cell) = buf.cell_mut((event_rect.x, y)) {
                    cell.set_symbol("▌")
                        .set_style(style.add_modifier(Modifier::BOLD));
                }
            }
        }

        // Draw current time line if today
        let now = Local::now();
        if is_today {
            let now_row = (now.hour() * 2 + now.minute() / 30) as usize;
            if now_row >= scroll && now_row < scroll + visible_rows {
                // Now line is visible
                let y = area.y + (now_row - scroll) as u16;
                draw_hline(
                    buf,
                    y,
                    area.x,
                    area.x + area.width,
                    "━",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                );
            } else {
                // Now is off-screen - show indicator
                let (indicator, y) = if now_row < scroll {
                    ("▲now", area.y)
                } else {
                    ("▼now", area.y + area.height.saturating_sub(1))
                };

                for (i, ch) in indicator.chars().enumerate() {
                    if area.x + (i as u16) < area.x + area.width {
                        if let Some(cell) = buf.cell_mut((area.x + i as u16, y)) {
                            cell.set_char(ch)
                                .set_style(Style::default().fg(Color::Red));
                        }
                    }
                }
            }
        }
    }
}
