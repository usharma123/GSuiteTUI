# Calendar Week View UI Improvements Plan

## Overview

This plan addresses 8 key improvements to make the calendar feel production-ready.
Organized by priority with exact file locations and code changes.

---

## Phase 1: Core Visual Polish (Do First)

### 1.1 Smarter Grid Lines (Hour vs Half-Hour)

**File:** `src/calendar/render.rs` - `render_day_events()`

**Current:** Lines 122-135 draw identical `─` for all rows

**Changes:**
```rust
// Replace the grid line logic at lines 128-134
if time_row % 2 == 0 {
    // Hour lines - strong
    let line = "─".repeat(area.width as usize);
    let para = Paragraph::new(line)
        .style(Style::default().fg(Color::Gray));
    f.render_widget(para, line_area);
} else {
    // Half-hour lines - subtle/dotted
    let line = "┄".repeat(area.width as usize);
    let para = Paragraph::new(line)
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(para, line_area);
}
```

### 1.2 Vertical Day Separators + Selected Day Background

**File:** `src/calendar/render.rs`

**Changes to `render_week()`:**
1. Track selected day index (add `selected_day: usize` to `CalendarState`)
2. Pass day index to render_day_column

**Add new helper function:**
```rust
fn fill_bg(f: &mut Frame, rect: Rect, color: Color) {
    let block = Block::default().style(Style::default().bg(color));
    f.render_widget(block, rect);
}
```

**Changes to `render_day_column()`:**
1. Draw vertical separator at column left edge
2. Fill background for selected/today column

```rust
// At start of render_day_column, before header
// Draw left border separator (except first column)
if day_idx > 0 {
    for y in area.y..area.y + area.height {
        f.buffer_mut().get_mut(area.x, y)
            .set_symbol("│")
            .set_style(Style::default().fg(Color::DarkGray));
    }
}

// Selected day background (subtle tint)
if is_selected || is_today {
    let bg_color = if is_selected {
        Color::Rgb(30, 35, 45) // subtle blue tint
    } else {
        Color::Rgb(25, 30, 35) // subtle today tint
    };
    fill_bg(f, area, bg_color);
}
```

### 1.3 Enhanced Now Line + Gutter Marker

**File:** `src/calendar/render.rs`

**Changes to `render_time_gutter()`** - Add marker at current time:
```rust
// After rendering time labels, add now marker
let now = Local::now();
let today = now.date_naive();
// Check if we're viewing current week
if self.week_start <= today && today <= self.week_end() {
    let now_row = (now.hour() * 2 + now.minute() / 30) as usize;
    let scroll = self.scroll_offset as usize;
    if now_row >= scroll && now_row < scroll + visible_rows {
        let y = area.y + (now_row - scroll) as u16;
        // Draw gutter marker
        f.buffer_mut().get_mut(area.x + area.width - 1, y)
            .set_symbol("▶")
            .set_style(Style::default().fg(Color::Red));
    }
}
```

**Changes to `render_day_events()`** - Improve now line (lines 195-206):
```rust
// Replace existing now line code
if now.date_naive() == date {
    let now_row = (now.hour() * 2 + now.minute() / 30) as usize;
    if now_row >= scroll && now_row < scroll + visible_rows {
        let y = area.y + (now_row - scroll) as u16;
        // Thicker, more visible line
        let line = "━".repeat(area.width as usize);
        let para = Paragraph::new(line)
            .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
        f.render_widget(para, Rect::new(area.x, y, area.width, 1));
    } else {
        // Now is off-screen - show indicator at top or bottom
        let indicator = if now_row < scroll { "▲ now" } else { "▼ now" };
        let y = if now_row < scroll { area.y } else { area.y + area.height - 1 };
        let para = Paragraph::new(indicator)
            .style(Style::default().fg(Color::Red));
        f.render_widget(para, Rect::new(area.x, y, 6, 1));
    }
}
```

---

## Phase 2: Event Card Improvements

### 2.1 Add unicode-width Dependency

**File:** `Cargo.toml`
```toml
unicode-width = "0.1"
```

### 2.2 Unicode-Safe Text Truncation

**File:** `src/calendar/render.rs` - Add helper function:
```rust
use unicode_width::UnicodeWidthStr;

fn clip_text(s: &str, max_width: usize) -> String {
    if UnicodeWidthStr::width(s) <= max_width {
        return s.to_string();
    }
    let mut out = String::new();
    let mut width = 0;
    for ch in s.chars() {
        let ch_width = UnicodeWidthStr::width(ch.to_string().as_str());
        if width + ch_width >= max_width.saturating_sub(1) {
            break;
        }
        out.push(ch);
        width += ch_width;
    }
    out.push('…');
    out
}
```

### 2.3 Event Card First Line: Time + Title

**File:** `src/calendar/render.rs` - `render_day_events()`

**Replace lines 177-183** (summary truncation):
```rust
// Build first line: HH:MM Title
let start_time = layout_event.event.start.time()
    .map(|t| format!("{:02}:{:02} ", t.hour(), t.minute()))
    .unwrap_or_default();

let available_width = (width as usize).saturating_sub(2); // padding
let time_width = UnicodeWidthStr::width(start_time.as_str());
let title_width = available_width.saturating_sub(time_width);

let display_text = format!("{}{}",
    start_time,
    clip_text(&layout_event.event.summary, title_width)
);
```

### 2.4 Minimum Event Height of 2 Rows

**File:** `src/calendar/layout.rs` - `time_to_rows()`

**Change line 78:**
```rust
// Ensure minimum 2 rows for visibility
let end_row = end_row.max(start_row + 2);
```

---

## Phase 3: Overlap Lane Improvements

### 3.1 Add 1-Cell Gap Between Lanes

**File:** `src/calendar/render.rs` - `render_day_events()` lines 155-158

**Replace lane width calculation:**
```rust
let gap = 1u16;
let total_gap = gap * (layout_event.total_lanes as u16 - 1);
let available = area.width.saturating_sub(total_gap);
let lane_width = available / layout_event.total_lanes as u16;
let x = area.x + (layout_event.lane as u16 * (lane_width + gap));
let width = lane_width.max(1);
```

### 3.2 Collapse to Stack Indicator When Too Narrow

**File:** `src/calendar/render.rs` - `render_day_events()`

**Add before rendering events in a lane:**
```rust
const MIN_LANE_WIDTH: u16 = 8;

// If lanes too narrow, render only first event + count badge
if lane_width < MIN_LANE_WIDTH && layout_event.total_lanes > 1 {
    // Only render if this is lane 0
    if layout_event.lane != 0 {
        continue;
    }
    // Count overlapping events at this time
    let overlap_count = layout_events.iter()
        .filter(|e| e.start_row == layout_event.start_row && e.total_lanes > 1)
        .count();

    // Render badge "+N" at top-right of event
    if overlap_count > 1 {
        let badge = format!("+{}", overlap_count - 1);
        // Render badge after event block...
    }
}
```

---

## Phase 4: All-Day Event Strip

### 4.1 Add All-Day Strip Area

**File:** `src/calendar/render.rs` - `render_day_column()`

**Change layout split (lines 105-108):**
```rust
// Split: header (1) + all-day strip (2-3) + time grid
let all_day_events = self.all_day_events_for_day(date);
let all_day_height = if all_day_events.is_empty() { 0 } else { 2.min(all_day_events.len() as u16) + 1 };

let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(1),              // Header
        Constraint::Length(all_day_height), // All-day strip
        Constraint::Min(1),                 // Time grid
    ])
    .split(area);

f.render_widget(header_para, chunks[0]);

if all_day_height > 0 {
    self.render_all_day_strip(f, chunks[1], &all_day_events);
}

self.render_day_events(f, chunks[2], date);
```

### 4.2 Add All-Day Strip Renderer

**File:** `src/calendar/render.rs` - New function:
```rust
fn render_all_day_strip(&self, f: &mut Frame, area: Rect, events: &[&CalendarEvent]) {
    // Background
    let block = Block::default()
        .style(Style::default().bg(Color::Rgb(40, 40, 50)));
    f.render_widget(block, area);

    // Render events as pills
    for (i, event) in events.iter().take(area.height as usize).enumerate() {
        let y = area.y + i as u16;
        let text = clip_text(&event.summary, area.width as usize - 2);
        let style = Style::default().bg(Color::Cyan).fg(Color::Black);
        let para = Paragraph::new(format!(" {} ", text)).style(style);
        f.render_widget(para, Rect::new(area.x, y, area.width, 1));
    }
}
```

---

## Phase 5: Interaction Polish

### 5.1 Add Selected Day to CalendarState

**File:** `src/calendar/model.rs`
```rust
pub struct CalendarState {
    // ... existing fields
    pub selected_day: usize,  // 0-6 (Mon-Sun)
    pub selected_event_idx: Option<usize>,  // Index within day's events
}

// Add navigation methods
impl CalendarState {
    pub fn select_prev_day(&mut self) {
        self.selected_day = self.selected_day.saturating_sub(1);
        self.selected_event_idx = None;
    }

    pub fn select_next_day(&mut self) {
        self.selected_day = (self.selected_day + 1).min(6);
        self.selected_event_idx = None;
    }

    pub fn select_prev_event(&mut self) {
        // Move to previous event by time
    }

    pub fn select_next_event(&mut self) {
        // Move to next event by time
    }
}
```

### 5.2 Update Keymap

**File:** `src/app/keymap.rs`

**Add to Action enum:**
```rust
SelectPrevDay,
SelectNextDay,
SelectPrevEvent,
SelectNextEvent,
```

**Update `map_calendar_key()`:**
```rust
// Day navigation
KeyCode::Left | KeyCode::Char('h') => Action::SelectPrevDay,
KeyCode::Right | KeyCode::Char('l') => Action::SelectNextDay,
// Event navigation
KeyCode::Up | KeyCode::Char('k') => Action::SelectPrevEvent,
KeyCode::Down | KeyCode::Char('j') => Action::SelectNextEvent,
// Week navigation (shift modifier)
KeyCode::Char('H') => Action::PrevWeek,
KeyCode::Char('L') => Action::NextWeek,
```

### 5.3 Selected Event Styling

**File:** `src/calendar/render.rs` - `render_day_events()`

```rust
// Check if this event is selected
let is_selected = self.selected_event_idx
    .map(|idx| layout_event.event.id == self.events_for_day(date)[idx].id)
    .unwrap_or(false);

let style = if is_selected {
    // Invert colors for selected event
    match layout_event.event.status {
        EventStatus::Confirmed => Style::default().bg(Color::White).fg(Color::Blue),
        // ... other statuses
    }
} else {
    // Normal styling
    match layout_event.event.status { /* ... */ }
};
```

### 5.4 Update Status Bar

**File:** `src/ui/status.rs`

Show keyboard hints for calendar:
```rust
if scene == Scene::CalendarWeek {
    hints = "h/l: day  j/k: event  H/L: week  g: now  /: search  Enter: details";
}
```

---

## Phase 6: Performance (Cached Layout)

### 6.1 Add Cached Layout Struct

**File:** `src/calendar/model.rs`
```rust
#[derive(Debug, Clone)]
pub struct CachedLayout {
    pub week_start: NaiveDate,
    pub scroll_offset: u16,
    pub area_hash: u64,  // Hash of render area dimensions
    pub placed_events: Vec<PlacedEvent>,
}

#[derive(Debug, Clone)]
pub struct PlacedEvent {
    pub event_id: String,
    pub rect: (u16, u16, u16, u16),  // x, y, w, h
    pub title_line: String,
    pub style: Style,
}
```

### 6.2 Cache Invalidation

**File:** `src/calendar/model.rs` - CalendarState
```rust
impl CalendarState {
    fn invalidate_cache(&mut self) {
        self.cached_layout = None;
    }

    // Call invalidate_cache() in:
    // - prev_week(), next_week()
    // - scroll_up(), scroll_down()
    // - When events change (after sync)
}
```

### 6.3 Use Cached Layout in Render

**File:** `src/calendar/render.rs`
```rust
fn render_day_events(&self, f: &mut Frame, area: Rect, date: NaiveDate) {
    // Check cache validity
    let cache_key = (self.week_start, self.scroll_offset, area.width, area.height);

    let placed_events = if let Some(ref cache) = self.cached_layout {
        if cache.matches(cache_key) {
            &cache.placed_events
        } else {
            // Recompute and cache
            self.recompute_layout(area)
        }
    } else {
        self.recompute_layout(area)
    };

    // Render from cache
    for placed in placed_events {
        // Direct buffer writes using cached positions
    }
}
```

---

## Implementation Order (Recommended)

### Immediate Impact (Do These 3 First):
1. **Vertical day separators + selected day background** (1.2)
2. **Now line + gutter marker** (1.3)
3. **Event first line = time+title with unicode truncation** (2.1-2.3)

### Second Pass:
4. Smarter grid lines (1.1)
5. Lane gap + stack indicator (3.1-3.2)
6. All-day strip (4.1-4.2)

### Polish:
7. Keyboard navigation (5.1-5.4)
8. Performance caching (6.1-6.3)

---

## Files to Modify

| File | Changes |
|------|---------|
| `Cargo.toml` | Add `unicode-width = "0.1"` |
| `src/calendar/model.rs` | Add `selected_day`, `cached_layout`, navigation methods |
| `src/calendar/render.rs` | Main rendering improvements (all phases) |
| `src/calendar/layout.rs` | Minimum 2-row height |
| `src/app/keymap.rs` | Day/event navigation actions |
| `src/engine/input.rs` | Handle new actions |
| `src/ui/status.rs` | Calendar keyboard hints |
