//! The Today view: one screen to start the day.
//!
//! Spec: top 3 tasks, today's events with countdown bars, and due items.
//! No more than 7 items per list by default. The UI shows "show more" when
//! the `*_more` counts are above 0.

use chrono::{DateTime, Duration, TimeZone, Utc};
use rusqlite::{params, Connection};
use serde::Serialize;

use super::tasks::{self, Task, TaskView};
use crate::db::{ids, DbError};

pub const LIST_LIMIT: usize = 7;
pub const TOP_COUNT: usize = 3;

#[derive(Debug, Serialize)]
pub struct TodayEvent {
    pub id: String,
    pub title: String,
    pub starts_at: String,
    pub ends_at: Option<String>,
    pub all_day: bool,
}

#[derive(Debug, Serialize)]
pub struct Today {
    /// Local date shown at the top, `YYYY-MM-DD`.
    pub date: String,
    pub top: Vec<Task>,
    pub due_today: Vec<Task>,
    pub due_today_more: usize,
    pub events: Vec<TodayEvent>,
    pub events_more: usize,
    /// How many tasks wait in Catch-up. Shown as a calm count, never a red list.
    pub catch_up_count: usize,
    pub inbox_count: usize,
}

pub fn build(conn: &Connection, now: DateTime<Utc>) -> Result<Today, DbError> {
    let today = ids::local_today(now);
    let today_str = today.format("%Y-%m-%d").to_string();

    // Active list is already sorted: soonest due first, then oldest.
    let active = tasks::list(conn, TaskView::Active, now)?;
    let top: Vec<Task> = active.iter().take(TOP_COUNT).cloned().collect();

    // Due today, minus anything already in the top 3.
    let due: Vec<Task> = active
        .iter()
        .filter(|t| t.data.due_on.as_deref() == Some(today_str.as_str()))
        .filter(|t| !top.iter().any(|x| x.id == t.id))
        .cloned()
        .collect();
    let due_today_more = due.len().saturating_sub(LIST_LIMIT);

    // Events that overlap today's local day.
    let day_start = ids::USER_TZ
        .from_local_datetime(&today.and_hms_opt(0, 0, 0).unwrap())
        .earliest()
        .unwrap()
        .with_timezone(&Utc);
    let day_end = day_start + Duration::days(1);
    let mut stmt = conn.prepare(
        "SELECT id, title, starts_at, ends_at, all_day FROM events
         WHERE starts_at < ?2 AND coalesce(ends_at, starts_at) >= ?1
         ORDER BY all_day DESC, starts_at",
    )?;
    let events: Vec<TodayEvent> = stmt
        .query_map(params![ids::to_iso(day_start), ids::to_iso(day_end)], |r| {
            Ok(TodayEvent {
                id: r.get(0)?,
                title: r.get(1)?,
                starts_at: r.get(2)?,
                ends_at: r.get(3)?,
                all_day: r.get::<_, i64>(4)? != 0,
            })
        })?
        .collect::<Result<_, _>>()?;
    let events_more = events.len().saturating_sub(LIST_LIMIT);

    Ok(Today {
        date: today_str,
        top,
        due_today: due.into_iter().take(LIST_LIMIT).collect(),
        due_today_more,
        events: events.into_iter().take(LIST_LIMIT).collect(),
        events_more,
        catch_up_count: tasks::list(conn, TaskView::CatchUp, now)?.len(),
        inbox_count: tasks::list(conn, TaskView::Inbox, now)?.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_conn;
    use crate::modules::tasks::{create, NewTask, TaskData};

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 6, 20, 0, 0).unwrap() // noon in Anchorage
    }

    fn task(c: &Connection, title: &str, due: Option<&str>) {
        create(
            c,
            &NewTask { title: title.into(), data: TaskData { due_on: due.map(Into::into), ..Default::default() }, ..Default::default() },
        )
        .unwrap();
    }

    #[test]
    fn top_three_and_limits() {
        let c = test_conn();
        task(&c, "Missed", Some("2026-10-01"));
        for i in 0..12 {
            task(&c, &format!("Due {i}"), Some("2026-10-06"));
        }
        task(&c, "Tomorrow", Some("2026-10-07"));
        let t = build(&c, now()).unwrap();
        assert_eq!(t.date, "2026-10-06");
        assert_eq!(t.top.len(), 3);
        assert_eq!(t.top[0].title, "Due 0");
        assert_eq!(t.due_today.len(), LIST_LIMIT);
        assert_eq!(t.due_today_more, 12 - 3 - LIST_LIMIT);
        assert_eq!(t.catch_up_count, 1);
    }

    #[test]
    fn events_for_local_day_only() {
        let c = test_conn();
        let ins = |title: &str, start: &str| {
            c.execute(
                "INSERT INTO events (id, title, starts_at, created_at, updated_at) VALUES (?1, ?2, ?3, 't', 't')",
                params![ids::new_id(), title, start],
            )
            .unwrap();
        };
        ins("Late last night", "2026-10-06T07:30:00Z"); // 23:30 Oct 5 local
        ins("Gym", "2026-10-06T16:00:00Z"); // 08:00 local
        ins("Dinner", "2026-10-07T03:00:00Z"); // 19:00 Oct 6 local
        let t = build(&c, now()).unwrap();
        let titles: Vec<_> = t.events.iter().map(|e| e.title.as_str()).collect();
        assert_eq!(titles, vec!["Gym", "Dinner"]);
    }
}
