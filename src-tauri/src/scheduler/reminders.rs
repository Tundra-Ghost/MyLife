//! Reminders: stored alerts that climb the ladder until you act.
//!
//! The agent calls `deliver` every 60 seconds. It returns the reminders
//! that should pop a desktop notification right now. Level 1 reminders
//! never notify; they only show in the Today view.

use chrono::{DateTime, Datelike, Duration, NaiveTime, TimeZone, Utc, Weekday};
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};

use super::{deliver_at, in_quiet_hours, ladder_level, LEVEL3_REPEAT};
use crate::db::ids::{self, USER_TZ};
use crate::db::DbError;
use crate::modules::calendar;

/// Spec: snoozing the same thing 5 times triggers a check-in question.
pub const CHECK_IN_AFTER_SNOOZES: i64 = 5;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Reminder {
    pub id: String,
    pub rule_id: Option<String>,
    pub item_id: Option<String>,
    pub title: String,
    pub fire_at: String,
    pub ladder_level: i64,
    pub max_ladder: i64,
    pub urgent: bool,
    pub snooze_count: i64,
    pub state: String,
    pub last_notified_at: Option<String>,
}

/// A reminder the agent should show as a desktop notification now.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Notice {
    pub reminder_id: String,
    pub title: String,
    pub level: u8,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Snooze {
    /// 3 hours from now.
    LaterToday,
    /// 9 AM tomorrow.
    Tomorrow,
    /// 9 AM Saturday (or next Saturday if it's the weekend).
    ThisWeekend,
    /// The next open 30 minutes on the calendar, 9 AM to 9 PM.
    WhenFree,
}

const SELECT: &str = "SELECT id, rule_id, item_id, title, fire_at, ladder_level, max_ladder, urgent, snooze_count, state, last_notified_at FROM reminders";
const OPEN_STATES: &str = "('pending','fired','snoozed')";

fn row(r: &Row) -> rusqlite::Result<Reminder> {
    Ok(Reminder {
        id: r.get(0)?,
        rule_id: r.get(1)?,
        item_id: r.get(2)?,
        title: r.get(3)?,
        fire_at: r.get(4)?,
        ladder_level: r.get(5)?,
        max_ladder: r.get(6)?,
        urgent: r.get::<_, i64>(7)? != 0,
        snooze_count: r.get(8)?,
        state: r.get(9)?,
        last_notified_at: r.get(10)?,
    })
}

fn parse(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s).map(|d| d.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now())
}

pub struct NewReminder<'a> {
    pub rule_id: Option<&'a str>,
    pub item_id: Option<&'a str>,
    pub title: &'a str,
    pub fire_at: DateTime<Utc>,
    pub max_ladder: u8,
    pub urgent: bool,
}

pub fn create(conn: &Connection, r: &NewReminder) -> Result<Reminder, DbError> {
    let id = ids::new_id();
    let now = ids::now_iso();
    conn.execute(
        "INSERT INTO reminders (id, rule_id, item_id, title, fire_at, max_ladder, urgent, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)",
        params![id, r.rule_id, r.item_id, r.title, ids::to_iso(r.fire_at), r.max_ladder, r.urgent as i64, now],
    )?;
    get(conn, &id)
}

pub fn get(conn: &Connection, id: &str) -> Result<Reminder, DbError> {
    conn.query_row(&format!("{SELECT} WHERE id = ?1"), params![id], row)
        .optional()?
        .ok_or(DbError::NotFound)
}

/// The open reminder a rule made for an item, if any.
pub fn open_for(conn: &Connection, rule_id: Option<&str>, item_id: &str) -> Result<Option<Reminder>, DbError> {
    Ok(conn
        .query_row(
            &format!("{SELECT} WHERE rule_id IS ?1 AND item_id = ?2 AND state IN {OPEN_STATES}"),
            params![rule_id, item_id],
            row,
        )
        .optional()?)
}

/// Moves an untouched reminder to a new time (for example, a task's due date changed).
pub fn reschedule(conn: &Connection, id: &str, fire_at: DateTime<Utc>, title: &str) -> Result<(), DbError> {
    conn.execute(
        "UPDATE reminders SET fire_at = ?2, title = ?3, ladder_level = 1, last_notified_at = NULL, state = 'pending', updated_at = ?4 WHERE id = ?1",
        params![id, ids::to_iso(fire_at), title, ids::now_iso()],
    )?;
    Ok(())
}

/// Closes every open reminder for an item (for example, the task was finished).
pub fn close_for_item(conn: &Connection, item_id: &str) -> Result<(), DbError> {
    conn.execute(
        &format!("UPDATE reminders SET state = 'done', updated_at = ?2 WHERE item_id = ?1 AND state IN {OPEN_STATES}"),
        params![item_id, ids::now_iso()],
    )?;
    Ok(())
}

/// Reminders that are due now: the Today view lists these.
pub fn active(conn: &Connection, now: DateTime<Utc>) -> Result<Vec<Reminder>, DbError> {
    let mut stmt = conn.prepare(&format!("{SELECT} WHERE state IN {OPEN_STATES} AND fire_at <= ?1 ORDER BY fire_at"))?;
    let rows = stmt.query_map(params![ids::to_iso(now)], row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// Decides which reminders notify now, and records that they did.
pub fn deliver(conn: &Connection, now: DateTime<Utc>) -> Result<Vec<Notice>, DbError> {
    let mut out = Vec::new();
    for r in active(conn, now)? {
        let fire_at = parse(&r.fire_at);
        // Quiet hours push the first nudge to 7 AM, unless urgent.
        if deliver_at(fire_at, r.urgent) > now {
            continue;
        }
        let level = ladder_level(fire_at, now, r.max_ladder.clamp(1, 4) as u8);
        if level < 2 {
            continue;
        }
        let due = match &r.last_notified_at {
            None => true,
            Some(last) => {
                let climbed = i64::from(level) > r.ladder_level;
                let repeat = level == 3 && now >= parse(last) + LEVEL3_REPEAT;
                (climbed || repeat) && (r.urgent || !in_quiet_hours(now))
            }
        };
        if !due {
            continue;
        }
        conn.execute(
            "UPDATE reminders SET ladder_level = ?2, last_notified_at = ?3, state = 'fired', updated_at = ?3 WHERE id = ?1",
            params![r.id, level, ids::to_iso(now)],
        )?;
        out.push(Notice { reminder_id: r.id, title: r.title, level });
    }
    Ok(out)
}

pub fn set_state(conn: &Connection, id: &str, state: &str) -> Result<(), DbError> {
    get(conn, id)?;
    conn.execute(
        "UPDATE reminders SET state = ?2, updated_at = ?3 WHERE id = ?1",
        params![id, state, ids::now_iso()],
    )?;
    Ok(())
}

/// Snoozes a reminder. Returns it so the UI can check `snooze_count`.
pub fn snooze(conn: &Connection, id: &str, option: Snooze, now: DateTime<Utc>) -> Result<Reminder, DbError> {
    get(conn, id)?;
    let when = snooze_until(conn, option, now)?;
    conn.execute(
        "UPDATE reminders SET fire_at = ?2, state = 'snoozed', ladder_level = 1, last_notified_at = NULL,
           snooze_count = snooze_count + 1, updated_at = ?3 WHERE id = ?1",
        params![id, ids::to_iso(when), ids::to_iso(now)],
    )?;
    get(conn, id)
}

/// Local midnight at the start of `date`, in UTC.
pub fn local_midnight(date: chrono::NaiveDate) -> DateTime<Utc> {
    local_at(date, 0, 0)
}

fn local_at(date: chrono::NaiveDate, h: u32, m: u32) -> DateTime<Utc> {
    USER_TZ
        .from_local_datetime(&date.and_time(NaiveTime::from_hms_opt(h, m, 0).unwrap()))
        .earliest()
        .unwrap()
        .with_timezone(&Utc)
}

pub fn snooze_until(conn: &Connection, option: Snooze, now: DateTime<Utc>) -> Result<DateTime<Utc>, DbError> {
    let today = ids::local_today(now);
    Ok(match option {
        Snooze::LaterToday => now + Duration::hours(3),
        Snooze::Tomorrow => local_at(today.succ_opt().unwrap(), 9, 0),
        Snooze::ThisWeekend => {
            let wd = today.weekday();
            let days = match wd {
                Weekday::Sat | Weekday::Sun => 7 - wd.num_days_from_monday() as i64 + 5, // next Saturday
                _ => 5 - wd.num_days_from_monday() as i64,
            };
            local_at(today + Duration::days(days), 9, 0)
        }
        Snooze::WhenFree => next_free_slot(conn, now, Duration::minutes(30))?,
    })
}

/// The next gap of `len` on the calendar, between 9 AM and 9 PM local, in the next 7 days.
/// Falls back to tomorrow 9 AM if the week is full.
pub fn next_free_slot(conn: &Connection, now: DateTime<Utc>, len: Duration) -> Result<DateTime<Utc>, DbError> {
    // Start at the next quarter hour.
    let mins = now.timestamp() / 60;
    let mut t = DateTime::from_timestamp(((mins / 15) + 1) * 15 * 60, 0).unwrap();
    let busy: Vec<(DateTime<Utc>, DateTime<Utc>)> = calendar::occurrences(conn, now, now + Duration::days(7))?
        .into_iter()
        .filter(|o| !o.all_day)
        .map(|o| {
            let s = parse(&o.starts_at);
            let e = o.ends_at.as_deref().map(parse).unwrap_or(s);
            (s, e)
        })
        .collect();
    let end = now + Duration::days(7);
    while t < end {
        let local = t.with_timezone(&USER_TZ);
        let day = local.date_naive();
        let (open, close) = (local_at(day, 9, 0), local_at(day, 21, 0));
        if t < open {
            t = open;
            continue;
        }
        if t + len > close {
            t = local_at(day.succ_opt().unwrap(), 9, 0);
            continue;
        }
        match busy.iter().filter(|(s, e)| *s < t + len && *e > t).map(|(_, e)| *e).max() {
            Some(e) => t = e,
            None => return Ok(t),
        }
    }
    Ok(local_at(ids::local_today(now).succ_opt().unwrap(), 9, 0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_conn;
    use crate::modules::calendar::{create as create_event, EventInput};

    fn ak(d: u32, h: u32, m: u32) -> DateTime<Utc> {
        USER_TZ.with_ymd_and_hms(2026, 10, d, h, m, 0).earliest().unwrap().with_timezone(&Utc)
    }

    fn new(c: &Connection, fire: DateTime<Utc>, max: u8, urgent: bool) -> Reminder {
        create(c, &NewReminder { rule_id: None, item_id: None, title: "Take meds", fire_at: fire, max_ladder: max, urgent }).unwrap()
    }

    #[test]
    fn nudges_once_then_escalates_and_repeats() {
        let c = test_conn();
        let r = new(&c, ak(6, 12, 0), 3, false);
        assert!(deliver(&c, ak(6, 11, 59)).unwrap().is_empty());
        // Level 2 at fire time, within the 60-second tick.
        let n = deliver(&c, ak(6, 12, 0)).unwrap();
        assert_eq!(n, vec![Notice { reminder_id: r.id.clone(), title: "Take meds".into(), level: 2 }]);
        // Not again on the next tick.
        assert!(deliver(&c, ak(6, 12, 1)).unwrap().is_empty());
        // Level 3 after 2 hours.
        assert_eq!(deliver(&c, ak(6, 14, 0)).unwrap()[0].level, 3);
        assert!(deliver(&c, ak(6, 17, 0)).unwrap().is_empty());
        // Level 3 repeats every 4 hours.
        assert_eq!(deliver(&c, ak(6, 18, 0)).unwrap().len(), 1);
    }

    #[test]
    fn default_ladder_stops_at_nudge() {
        let c = test_conn();
        new(&c, ak(6, 12, 0), 2, false);
        assert_eq!(deliver(&c, ak(6, 12, 0)).unwrap().len(), 1);
        assert!(deliver(&c, ak(6, 20, 0)).unwrap().is_empty());
    }

    #[test]
    fn quiet_level_never_notifies_but_shows_in_today() {
        let c = test_conn();
        new(&c, ak(6, 12, 0), 1, false);
        assert!(deliver(&c, ak(6, 12, 0)).unwrap().is_empty());
        assert_eq!(active(&c, ak(6, 12, 0)).unwrap().len(), 1);
    }

    #[test]
    fn quiet_hours_wait_unless_urgent() {
        let c = test_conn();
        new(&c, ak(6, 23, 0), 2, false);
        let urgent = new(&c, ak(6, 23, 0), 2, true);
        let n = deliver(&c, ak(6, 23, 0)).unwrap();
        assert_eq!(n.len(), 1);
        assert_eq!(n[0].reminder_id, urgent.id);
        // The other one waits until 7 AM.
        assert!(deliver(&c, ak(7, 6, 59)).unwrap().is_empty());
        assert_eq!(deliver(&c, ak(7, 7, 0)).unwrap().len(), 1);
    }

    #[test]
    fn missed_while_asleep_fires_once() {
        let c = test_conn();
        new(&c, ak(6, 9, 0), 2, false);
        // PC wakes at 3 PM. One notification, not a backlog.
        assert_eq!(deliver(&c, ak(6, 15, 0)).unwrap().len(), 1);
        assert!(deliver(&c, ak(6, 15, 1)).unwrap().is_empty());
    }

    #[test]
    fn snooze_options() {
        let c = test_conn();
        let now = ak(6, 14, 0); // Tuesday 2 PM
        assert_eq!(snooze_until(&c, Snooze::LaterToday, now).unwrap(), ak(6, 17, 0));
        assert_eq!(snooze_until(&c, Snooze::Tomorrow, now).unwrap(), ak(7, 9, 0));
        assert_eq!(snooze_until(&c, Snooze::ThisWeekend, now).unwrap(), ak(10, 9, 0));
        // On Saturday, "this weekend" means next Saturday.
        assert_eq!(snooze_until(&c, Snooze::ThisWeekend, ak(10, 10, 0)).unwrap(), ak(17, 9, 0));
    }

    #[test]
    fn when_free_skips_busy_time() {
        let c = test_conn();
        let ev = |s: DateTime<Utc>, e: DateTime<Utc>| EventInput {
            title: "busy".into(),
            starts_at: ids::to_iso(s),
            ends_at: Some(ids::to_iso(e)),
            all_day: false,
            rrule: None,
            item_id: None,
        };
        create_event(&c, &ev(ak(6, 14, 0), ak(6, 15, 0))).unwrap();
        create_event(&c, &ev(ak(6, 15, 0), ak(6, 16, 30))).unwrap();
        assert_eq!(snooze_until(&c, Snooze::WhenFree, ak(6, 13, 50)).unwrap(), ak(6, 16, 30));
        // Late at night: first slot is 9 AM tomorrow.
        assert_eq!(snooze_until(&c, Snooze::WhenFree, ak(6, 22, 0)).unwrap(), ak(7, 9, 0));
    }

    #[test]
    fn snooze_resets_ladder_and_counts() {
        let c = test_conn();
        let r = new(&c, ak(6, 12, 0), 3, false);
        deliver(&c, ak(6, 12, 0)).unwrap();
        let r = snooze(&c, &r.id, Snooze::LaterToday, ak(6, 12, 5)).unwrap();
        assert_eq!(r.snooze_count, 1);
        assert_eq!(r.ladder_level, 1);
        assert!(active(&c, ak(6, 12, 6)).unwrap().is_empty());
        assert_eq!(deliver(&c, ak(6, 15, 5)).unwrap().len(), 1);
    }
}
