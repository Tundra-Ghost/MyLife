//! Calendar: events with optional repeats, and time-blocking tasks.
//!
//! An event is a row in `events`. Repeating events store an RRULE and are
//! expanded into occurrences when a date range is read. Google sync
//! (two-way) builds on this in the next step.

use chrono::{DateTime, Duration, Utc};
use rrule::{RRuleSet, Tz as RTz};
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};

use super::{tasks, timeline};
use crate::db::{ids, DbError};

/// Length of a time block when a task has no estimate.
pub const DEFAULT_BLOCK_MINUTES: i64 = 30;

/// Hard cap on occurrences returned for one range, so a bad rule can't hang the UI.
const MAX_OCCURRENCES: u16 = 2000;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Event {
    pub id: String,
    pub item_id: Option<String>,
    pub title: String,
    pub starts_at: String,
    pub ends_at: Option<String>,
    pub all_day: bool,
    pub rrule: Option<String>,
    pub source: String,
}

/// One time an event happens. For a repeating event there are many.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Occurrence {
    pub event_id: String,
    pub item_id: Option<String>,
    pub title: String,
    pub starts_at: String,
    pub ends_at: Option<String>,
    pub all_day: bool,
    pub repeats: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventInput {
    pub title: String,
    pub starts_at: String,
    pub ends_at: Option<String>,
    #[serde(default)]
    pub all_day: bool,
    pub rrule: Option<String>,
    pub item_id: Option<String>,
}

fn parse_utc(s: &str) -> Result<DateTime<Utc>, DbError> {
    DateTime::parse_from_rfc3339(s)
        .map(|d| d.with_timezone(&Utc))
        .map_err(|_| DbError::Invalid(format!("Bad time: {s}")))
}

fn row_to_event(r: &Row) -> rusqlite::Result<Event> {
    Ok(Event {
        id: r.get(0)?,
        item_id: r.get(1)?,
        title: r.get(2)?,
        starts_at: r.get(3)?,
        ends_at: r.get(4)?,
        all_day: r.get::<_, i64>(5)? != 0,
        rrule: r.get(6)?,
        source: r.get(7)?,
    })
}

const SELECT_EVENT: &str =
    "SELECT id, item_id, title, starts_at, ends_at, all_day, rrule, source FROM events WHERE archived_at IS NULL";

/// Checks input and returns it with times normalized to our stored format.
fn clean(input: &EventInput) -> Result<(String, Option<String>), DbError> {
    if input.title.trim().is_empty() {
        return Err(DbError::Invalid("Title can't be empty.".into()));
    }
    let start = parse_utc(&input.starts_at)?;
    let end = input.ends_at.as_deref().map(parse_utc).transpose()?;
    if matches!(end, Some(e) if e < start) {
        return Err(DbError::Invalid("End must be after start.".into()));
    }
    if let Some(r) = &input.rrule {
        crate::scheduler::next_after(r, start, start).map_err(|e| DbError::Invalid(format!("Bad repeat rule: {e}")))?;
    }
    Ok((ids::to_iso(start), end.map(ids::to_iso)))
}

pub fn create(conn: &Connection, input: &EventInput) -> Result<Event, DbError> {
    let (start, end) = clean(input)?;
    let id = ids::new_id();
    let now = ids::now_iso();
    conn.execute(
        "INSERT INTO events (id, item_id, title, starts_at, ends_at, all_day, rrule, source, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'local', ?8, ?8)",
        params![id, input.item_id, input.title.trim(), start, end, input.all_day as i64, input.rrule, now],
    )?;
    timeline::log(conn, input.item_id.as_deref(), "event_created", None)?;
    get(conn, &id)
}

pub fn get(conn: &Connection, id: &str) -> Result<Event, DbError> {
    conn.query_row(&format!("{SELECT_EVENT} AND id = ?1"), params![id], row_to_event)
        .optional()?
        .ok_or(DbError::NotFound)
}

/// Replaces an event's fields. Changing a repeating event changes every occurrence.
pub fn update(conn: &Connection, id: &str, input: &EventInput) -> Result<Event, DbError> {
    get(conn, id)?;
    let (start, end) = clean(input)?;
    conn.execute(
        "UPDATE events SET title = ?2, starts_at = ?3, ends_at = ?4, all_day = ?5, rrule = ?6, item_id = ?7, updated_at = ?8
         WHERE id = ?1",
        params![id, input.title.trim(), start, end, input.all_day as i64, input.rrule, input.item_id, ids::now_iso()],
    )?;
    get(conn, id)
}

/// Soft delete.
pub fn archive(conn: &Connection, id: &str) -> Result<(), DbError> {
    let ev = get(conn, id)?;
    let now = ids::now_iso();
    conn.execute("UPDATE events SET archived_at = ?2, updated_at = ?2 WHERE id = ?1", params![id, now])?;
    timeline::log(conn, ev.item_id.as_deref(), "event_archived", None)?;
    Ok(())
}

/// Every occurrence that overlaps `[from, to)`, sorted by start. All-day first on ties.
pub fn occurrences(conn: &Connection, from: DateTime<Utc>, to: DateTime<Utc>) -> Result<Vec<Occurrence>, DbError> {
    // Single events are filtered in SQL. Repeating ones are expanded in Rust.
    let mut stmt = conn.prepare(&format!(
        "{SELECT_EVENT} AND (rrule IS NOT NULL OR (starts_at < ?2 AND coalesce(ends_at, starts_at) >= ?1))"
    ))?;
    let events: Vec<Event> = stmt
        .query_map(params![ids::to_iso(from), ids::to_iso(to)], row_to_event)?
        .collect::<Result<_, _>>()?;

    let mut out = Vec::new();
    for ev in events {
        let start = parse_utc(&ev.starts_at)?;
        let length = ev.ends_at.as_deref().map(parse_utc).transpose()?.map(|e| e - start);
        let make = |s: DateTime<Utc>| Occurrence {
            event_id: ev.id.clone(),
            item_id: ev.item_id.clone(),
            title: ev.title.clone(),
            starts_at: ids::to_iso(s),
            ends_at: length.map(|l| ids::to_iso(s + l)),
            all_day: ev.all_day,
            repeats: ev.rrule.is_some(),
        };
        match &ev.rrule {
            None => out.push(make(start)),
            Some(rule) => {
                // Look back by the event's length so one that started before `from` still shows.
                let look_from = from - length.unwrap_or_else(Duration::zero);
                for s in expand(rule, start, look_from, to)? {
                    if s + length.unwrap_or_else(Duration::zero) >= from && s < to {
                        out.push(make(s));
                    }
                }
            }
        }
    }
    out.sort_by(|a, b| a.starts_at.cmp(&b.starts_at).then(b.all_day.cmp(&a.all_day)));
    Ok(out)
}

/// RRULE start times in `[from, to)`, in the user's time zone.
fn expand(rule: &str, dtstart: DateTime<Utc>, from: DateTime<Utc>, to: DateTime<Utc>) -> Result<Vec<DateTime<Utc>>, DbError> {
    let local = dtstart.with_timezone(&ids::USER_TZ);
    let text = format!(
        "DTSTART;TZID={}:{}\nRRULE:{}",
        ids::USER_TZ.name(),
        local.format("%Y%m%dT%H%M%S"),
        rule.trim_start_matches("RRULE:")
    );
    let set: RRuleSet = text.parse().map_err(|e| DbError::Invalid(format!("Bad repeat rule: {e}")))?;
    Ok(set
        .after(from.with_timezone(&RTz::UTC))
        .before(to.with_timezone(&RTz::UTC))
        .all(MAX_OCCURRENCES)
        .dates
        .into_iter()
        .map(|d| d.with_timezone(&Utc))
        .filter(|d| *d < to)
        .collect())
}

/// Time-blocks a task: puts it on the calendar at `starts_at` for its
/// estimate (or 30 minutes). The event links back to the task.
pub fn block_task(conn: &Connection, task_id: &str, starts_at: &str) -> Result<Event, DbError> {
    let task = tasks::get(conn, task_id)?;
    let start = parse_utc(starts_at)?;
    let minutes = task.est_minutes.unwrap_or(DEFAULT_BLOCK_MINUTES);
    create(
        conn,
        &EventInput {
            title: task.title,
            starts_at: ids::to_iso(start),
            ends_at: Some(ids::to_iso(start + Duration::minutes(minutes))),
            all_day: false,
            rrule: None,
            item_id: Some(task.id),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_conn;
    use crate::modules::tasks::{create as create_task, NewTask};
    use chrono::TimeZone;

    fn ak(d: u32, h: u32) -> DateTime<Utc> {
        ids::USER_TZ.with_ymd_and_hms(2026, 10, d, h, 0, 0).earliest().unwrap().with_timezone(&Utc)
    }

    fn input(title: &str, start: DateTime<Utc>, hours: i64, rrule: Option<&str>) -> EventInput {
        EventInput {
            title: title.into(),
            starts_at: ids::to_iso(start),
            ends_at: Some(ids::to_iso(start + Duration::hours(hours))),
            all_day: false,
            rrule: rrule.map(Into::into),
            item_id: None,
        }
    }

    #[test]
    fn create_edit_archive() {
        let c = test_conn();
        let e = create(&c, &input("Dentist", ak(7, 15), 1, None)).unwrap();
        assert_eq!(e.starts_at, "2026-10-07T23:00:00Z");
        let e = update(&c, &e.id, &input("Dentist (moved)", ak(8, 15), 1, None)).unwrap();
        assert_eq!(e.title, "Dentist (moved)");
        archive(&c, &e.id).unwrap();
        assert!(occurrences(&c, ak(1, 0), ak(31, 0)).unwrap().is_empty());
        assert!(matches!(get(&c, &e.id), Err(DbError::NotFound)));
    }

    #[test]
    fn rejects_bad_input() {
        let c = test_conn();
        assert!(create(&c, &input(" ", ak(7, 9), 1, None)).is_err());
        assert!(create(&c, &input("x", ak(7, 9), -1, None)).is_err());
        assert!(create(&c, &input("x", ak(7, 9), 1, Some("FREQ=NEVER"))).is_err());
        let mut bad = input("x", ak(7, 9), 1, None);
        bad.starts_at = "tomorrow".into();
        assert!(create(&c, &bad).is_err());
    }

    #[test]
    fn range_includes_overlaps_only() {
        let c = test_conn();
        create(&c, &input("Before", ak(5, 9), 1, None)).unwrap();
        create(&c, &input("Spans midnight", ak(5, 23), 2, None)).unwrap();
        create(&c, &input("Inside", ak(6, 12), 1, None)).unwrap();
        create(&c, &input("After", ak(7, 0), 1, None)).unwrap();
        let titles: Vec<_> = occurrences(&c, ak(6, 0), ak(7, 0)).unwrap().into_iter().map(|o| o.title).collect();
        assert_eq!(titles, vec!["Spans midnight", "Inside"]);
    }

    #[test]
    fn repeating_events_expand() {
        let c = test_conn();
        // Gym Mon/Wed/Fri at 6 AM, from Oct 5 2026 (a Monday).
        create(&c, &input("Gym", ak(5, 6), 1, Some("FREQ=WEEKLY;BYDAY=MO,WE,FR"))).unwrap();
        let occ = occurrences(&c, ak(5, 0), ak(12, 0)).unwrap();
        let starts: Vec<_> = occ.iter().map(|o| o.starts_at.as_str()).collect();
        assert_eq!(starts, vec!["2026-10-05T14:00:00Z", "2026-10-07T14:00:00Z", "2026-10-09T14:00:00Z"]);
        assert!(occ.iter().all(|o| o.repeats && o.ends_at.is_some()));
        // Nothing before the first one.
        assert!(occurrences(&c, ak(1, 0), ak(5, 0)).unwrap().is_empty());
    }

    #[test]
    fn time_block_a_task() {
        let c = test_conn();
        let t = create_task(&c, &NewTask { title: "Taxes".into(), est_minutes: Some(90), ..Default::default() }).unwrap();
        let e = block_task(&c, &t.id, &ids::to_iso(ak(10, 9))).unwrap();
        assert_eq!(e.item_id.as_deref(), Some(t.id.as_str()));
        assert_eq!(e.ends_at.as_deref(), Some(ids::to_iso(ak(10, 9) + Duration::minutes(90)).as_str()));
        let u = create_task(&c, &NewTask { title: "Quick".into(), ..Default::default() }).unwrap();
        let e = block_task(&c, &u.id, &ids::to_iso(ak(10, 11))).unwrap();
        assert_eq!(e.ends_at.as_deref(), Some(ids::to_iso(ak(10, 11) + Duration::minutes(30)).as_str()));
    }
}
