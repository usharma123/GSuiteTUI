use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, NaiveTime, Timelike, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: String,
    pub summary: String,
    pub start: EventTime,
    pub end: EventTime,
    pub status: EventStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventTime {
    DateTime(DateTime<Utc>),
    Date(NaiveDate),
}

impl EventTime {
    pub fn date(&self) -> NaiveDate {
        match self {
            EventTime::DateTime(dt) => dt.date_naive(),
            EventTime::Date(d) => *d,
        }
    }

    pub fn time(&self) -> Option<NaiveTime> {
        match self {
            EventTime::DateTime(dt) => Some(dt.time()),
            EventTime::Date(_) => None,
        }
    }

    pub fn is_all_day(&self) -> bool {
        matches!(self, EventTime::Date(_))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventStatus {
    Confirmed,
    Tentative,
    Cancelled,
}

impl Default for EventStatus {
    fn default() -> Self {
        EventStatus::Confirmed
    }
}

#[derive(Debug)]
pub struct CalendarState {
    pub events: Vec<CalendarEvent>,
    pub week_start: NaiveDate,
    pub scroll_offset: u16,
    pub sync_token: Option<String>,
    pub selected_event: Option<usize>,
}

impl CalendarState {
    pub fn new() -> Self {
        let today = Local::now().date_naive();
        let week_start = Self::week_start_for(today);
        Self {
            events: Vec::new(),
            week_start,
            scroll_offset: 8 * 2, // Start at 8 AM (2 rows per hour)
            sync_token: None,
            selected_event: None,
        }
    }

    pub fn mock() -> Self {
        let today = Local::now().date_naive();
        let week_start = Self::week_start_for(today);

        // Create some mock events with overlaps
        let events = vec![
            CalendarEvent {
                id: "1".to_string(),
                summary: "Morning Standup".to_string(),
                start: EventTime::DateTime(
                    today
                        .and_hms_opt(9, 0, 0)
                        .unwrap()
                        .and_local_timezone(Local)
                        .unwrap()
                        .to_utc(),
                ),
                end: EventTime::DateTime(
                    today
                        .and_hms_opt(9, 30, 0)
                        .unwrap()
                        .and_local_timezone(Local)
                        .unwrap()
                        .to_utc(),
                ),
                status: EventStatus::Confirmed,
            },
            CalendarEvent {
                id: "2".to_string(),
                summary: "Team Meeting".to_string(),
                start: EventTime::DateTime(
                    today
                        .and_hms_opt(9, 15, 0)
                        .unwrap()
                        .and_local_timezone(Local)
                        .unwrap()
                        .to_utc(),
                ),
                end: EventTime::DateTime(
                    today
                        .and_hms_opt(10, 15, 0)
                        .unwrap()
                        .and_local_timezone(Local)
                        .unwrap()
                        .to_utc(),
                ),
                status: EventStatus::Confirmed,
            },
            CalendarEvent {
                id: "3".to_string(),
                summary: "Lunch".to_string(),
                start: EventTime::DateTime(
                    today
                        .and_hms_opt(12, 0, 0)
                        .unwrap()
                        .and_local_timezone(Local)
                        .unwrap()
                        .to_utc(),
                ),
                end: EventTime::DateTime(
                    today
                        .and_hms_opt(13, 0, 0)
                        .unwrap()
                        .and_local_timezone(Local)
                        .unwrap()
                        .to_utc(),
                ),
                status: EventStatus::Confirmed,
            },
            CalendarEvent {
                id: "4".to_string(),
                summary: "1:1 with Manager".to_string(),
                start: EventTime::DateTime(
                    today
                        .and_hms_opt(14, 0, 0)
                        .unwrap()
                        .and_local_timezone(Local)
                        .unwrap()
                        .to_utc(),
                ),
                end: EventTime::DateTime(
                    today
                        .and_hms_opt(14, 30, 0)
                        .unwrap()
                        .and_local_timezone(Local)
                        .unwrap()
                        .to_utc(),
                ),
                status: EventStatus::Confirmed,
            },
            CalendarEvent {
                id: "5".to_string(),
                summary: "Project Review".to_string(),
                start: EventTime::DateTime(
                    today
                        .and_hms_opt(14, 0, 0)
                        .unwrap()
                        .and_local_timezone(Local)
                        .unwrap()
                        .to_utc(),
                ),
                end: EventTime::DateTime(
                    today
                        .and_hms_opt(15, 0, 0)
                        .unwrap()
                        .and_local_timezone(Local)
                        .unwrap()
                        .to_utc(),
                ),
                status: EventStatus::Tentative,
            },
            // All-day event tomorrow
            CalendarEvent {
                id: "6".to_string(),
                summary: "Holiday".to_string(),
                start: EventTime::Date(today + Duration::days(1)),
                end: EventTime::Date(today + Duration::days(2)),
                status: EventStatus::Confirmed,
            },
        ];

        Self {
            events,
            week_start,
            scroll_offset: 8 * 2,
            sync_token: None,
            selected_event: None,
        }
    }

    fn week_start_for(date: NaiveDate) -> NaiveDate {
        let days_since_monday = date.weekday().num_days_from_monday();
        date - Duration::days(days_since_monday as i64)
    }

    pub fn week_end(&self) -> NaiveDate {
        self.week_start + Duration::days(6)
    }

    pub fn prev_week(&mut self) {
        self.week_start -= Duration::days(7);
    }

    pub fn next_week(&mut self) {
        self.week_start += Duration::days(7);
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(2);
    }

    pub fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(2).min(48 - 10);
    }

    pub fn jump_to_now(&mut self) {
        let now = Local::now();
        self.week_start = Self::week_start_for(now.date_naive());
        let hour = now.hour() as u16;
        self.scroll_offset = (hour * 2).saturating_sub(4).min(48 - 10);
    }

    pub fn events_for_day(&self, date: NaiveDate) -> Vec<&CalendarEvent> {
        self.events
            .iter()
            .filter(|e| {
                let start_date = e.start.date();
                let end_date = e.end.date();
                date >= start_date && date < end_date
                    || (date == start_date)
                    || (date == end_date && !e.start.is_all_day())
            })
            .collect()
    }

    pub fn all_day_events_for_day(&self, date: NaiveDate) -> Vec<&CalendarEvent> {
        self.events
            .iter()
            .filter(|e| e.start.is_all_day() && e.start.date() <= date && e.end.date() > date)
            .collect()
    }

    pub fn timed_events_for_day(&self, date: NaiveDate) -> Vec<&CalendarEvent> {
        self.events
            .iter()
            .filter(|e| !e.start.is_all_day() && e.start.date() == date)
            .collect()
    }
}

impl Default for CalendarState {
    fn default() -> Self {
        Self::new()
    }
}
