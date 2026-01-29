use chrono::{NaiveDate, NaiveTime};
use chrono::Timelike;

use super::model::{CalendarEvent, EventTime};

#[derive(Debug, Clone)]
pub struct LayoutEvent<'a> {
    pub event: &'a CalendarEvent,
    pub lane: usize,
    pub total_lanes: usize,
    pub day_col: usize,
    pub start_row: usize,
    pub end_row: usize,
}


pub fn compute_layout<'a>(events: &'a [CalendarEvent], day: NaiveDate) -> Vec<LayoutEvent<'a>> {
    // Filter events for this day and convert to time slots
    let mut slots: Vec<(usize, &CalendarEvent, usize, usize)> = events
        .iter()
        .enumerate()
        .filter_map(|(idx, event)| {
            if event.start.is_all_day() {
                return None;
            }
            if event.start.date() != day {
                return None;
            }
            let (start_row, end_row) = time_to_rows(&event.start, &event.end);
            Some((idx, event, start_row, end_row))
        })
        .collect();

    // Sort by start time, then by duration (longer first)
    slots.sort_by(|a, b| {
        a.2.cmp(&b.2)
            .then_with(|| (b.3 - b.2).cmp(&(a.3 - a.2)))
    });

    if slots.is_empty() {
        return vec![];
    }

    // Build overlap groups using union-find
    let mut groups = build_overlap_groups(&slots);

    // For each group, assign lanes using greedy coloring
    let mut result = Vec::new();

    for group in &mut groups {
        let lanes = assign_lanes_greedy(group);
        let total_lanes = lanes.iter().max().map(|&l| l + 1).unwrap_or(1);

        for (i, &(_idx, event, start_row, end_row)) in group.iter().enumerate() {
            result.push(LayoutEvent {
                event,
                lane: lanes[i],
                total_lanes,
                day_col: 0, // Will be set by render
                start_row,
                end_row,
            });
        }
    }

    result
}

fn time_to_rows(start: &EventTime, end: &EventTime) -> (usize, usize) {
    let start_time = start.time().unwrap_or(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    let end_time = end.time().unwrap_or(NaiveTime::from_hms_opt(23, 59, 0).unwrap());

    // 2 rows per hour
    let start_row = (start_time.hour() * 2 + start_time.minute() / 30) as usize;
    let end_row = (end_time.hour() * 2 + (end_time.minute() + 29) / 30) as usize;

    // Ensure minimum 1 row
    let end_row = end_row.max(start_row + 1);

    (start_row, end_row.min(48))
}

fn build_overlap_groups<'a>(
    slots: &[(usize, &'a CalendarEvent, usize, usize)],
) -> Vec<Vec<(usize, &'a CalendarEvent, usize, usize)>> {
    if slots.is_empty() {
        return vec![];
    }

    // Union-find parent array
    let n = slots.len();
    let mut parent: Vec<usize> = (0..n).collect();

    fn find(parent: &mut [usize], x: usize) -> usize {
        if parent[x] != x {
            parent[x] = find(parent, parent[x]);
        }
        parent[x]
    }

    fn union(parent: &mut [usize], x: usize, y: usize) {
        let px = find(parent, x);
        let py = find(parent, y);
        if px != py {
            parent[px] = py;
        }
    }

    // Union overlapping events
    for i in 0..n {
        for j in (i + 1)..n {
            let (_, _, start_i, end_i) = slots[i];
            let (_, _, start_j, end_j) = slots[j];

            // Check if events overlap
            if start_i < end_j && start_j < end_i {
                union(&mut parent, i, j);
            }
        }
    }

    // Group by root
    let mut group_map: std::collections::HashMap<usize, Vec<usize>> =
        std::collections::HashMap::new();
    for i in 0..n {
        let root = find(&mut parent, i);
        group_map.entry(root).or_default().push(i);
    }

    // Convert to result
    group_map
        .into_values()
        .map(|indices| {
            let mut group: Vec<_> = indices.into_iter().map(|i| slots[i]).collect();
            // Sort within group by start time
            group.sort_by_key(|&(_, _, start, _)| start);
            group
        })
        .collect()
}

fn assign_lanes_greedy(events: &[(usize, &CalendarEvent, usize, usize)]) -> Vec<usize> {
    let mut lane_ends: Vec<usize> = vec![];
    let mut lanes = Vec::with_capacity(events.len());

    for &(_, _, start, end) in events {
        // Find first lane where start >= lane_end
        let lane = lane_ends
            .iter()
            .position(|&e| start >= e)
            .unwrap_or_else(|| {
                lane_ends.push(0);
                lane_ends.len() - 1
            });
        lane_ends[lane] = end;
        lanes.push(lane);
    }

    lanes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendar::model::EventStatus;
    use chrono::{Datelike, Local, TimeZone};

    fn make_event(id: &str, start_hour: u32, start_min: u32, end_hour: u32, end_min: u32) -> CalendarEvent {
        let today = Local::now().date_naive();
        CalendarEvent {
            id: id.to_string(),
            summary: format!("Event {}", id),
            start: EventTime::DateTime(
                Local
                    .with_ymd_and_hms(today.year(), today.month(), today.day(), start_hour, start_min, 0)
                    .unwrap()
                    .to_utc(),
            ),
            end: EventTime::DateTime(
                Local
                    .with_ymd_and_hms(today.year(), today.month(), today.day(), end_hour, end_min, 0)
                    .unwrap()
                    .to_utc(),
            ),
            status: EventStatus::Confirmed,
        }
    }

    #[test]
    fn test_no_overlap_single_lane() {
        let today = Local::now().date_naive();
        let events = vec![
            make_event("1", 9, 0, 10, 0),
            make_event("2", 11, 0, 12, 0),
            make_event("3", 14, 0, 15, 0),
        ];

        let layout = compute_layout(&events, today);

        assert_eq!(layout.len(), 3);
        for l in &layout {
            assert_eq!(l.total_lanes, 1, "Expected 1 lane for non-overlapping events");
            assert_eq!(l.lane, 0);
        }
    }

    #[test]
    fn test_two_overlapping_events() {
        let today = Local::now().date_naive();
        let events = vec![
            make_event("1", 9, 0, 10, 0),
            make_event("2", 9, 30, 10, 30),
        ];

        let layout = compute_layout(&events, today);

        assert_eq!(layout.len(), 2);
        assert_eq!(layout[0].total_lanes, 2);
        assert_eq!(layout[1].total_lanes, 2);
        // They should be in different lanes
        assert_ne!(layout[0].lane, layout[1].lane);
    }

    #[test]
    fn test_chain_overlap_two_lanes() {
        // A overlaps B, B overlaps C, but A doesn't overlap C
        let today = Local::now().date_naive();
        let events = vec![
            make_event("A", 9, 0, 10, 0),   // 9:00-10:00
            make_event("B", 9, 30, 10, 30), // 9:30-10:30 (overlaps A and C)
            make_event("C", 10, 0, 11, 0),  // 10:00-11:00 (doesn't overlap A)
        ];

        let layout = compute_layout(&events, today);

        assert_eq!(layout.len(), 3);
        // All should be in same group with 2 lanes
        for l in &layout {
            assert_eq!(l.total_lanes, 2, "Expected 2 lanes for chain overlap");
        }

        // A and C should be in same lane (they don't overlap)
        let a_layout = layout.iter().find(|l| l.event.id == "A").unwrap();
        let c_layout = layout.iter().find(|l| l.event.id == "C").unwrap();
        assert_eq!(a_layout.lane, c_layout.lane, "A and C should share a lane");
    }

    #[test]
    fn test_three_overlapping_events() {
        let today = Local::now().date_naive();
        let events = vec![
            make_event("1", 9, 0, 10, 0),
            make_event("2", 9, 0, 10, 0),
            make_event("3", 9, 0, 10, 0),
        ];

        let layout = compute_layout(&events, today);

        assert_eq!(layout.len(), 3);
        for l in &layout {
            assert_eq!(l.total_lanes, 3, "Expected 3 lanes");
        }
        // All in different lanes
        let lanes: std::collections::HashSet<_> = layout.iter().map(|l| l.lane).collect();
        assert_eq!(lanes.len(), 3, "All events should be in different lanes");
    }

    #[test]
    fn test_empty_events() {
        let today = Local::now().date_naive();
        let events: Vec<CalendarEvent> = vec![];
        let layout = compute_layout(&events, today);
        assert!(layout.is_empty());
    }

    #[test]
    fn test_all_day_events_excluded() {
        let today = Local::now().date_naive();
        let events = vec![CalendarEvent {
            id: "allday".to_string(),
            summary: "All Day".to_string(),
            start: EventTime::Date(today),
            end: EventTime::Date(today + chrono::Duration::days(1)),
            status: EventStatus::Confirmed,
        }];

        let layout = compute_layout(&events, today);
        assert!(layout.is_empty(), "All-day events should be excluded from layout");
    }
}
