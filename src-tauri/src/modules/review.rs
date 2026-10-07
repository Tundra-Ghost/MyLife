//! Daily brief (morning) and evening shutdown.
//!
//! The brief ends with one question: "What is your one must-do today?"
//! The shutdown reviews what got done, moves what didn't, and picks
//! tomorrow's top 3. Both choices pin tasks to the top of the Today view.

use chrono::{DateTime, Duration, NaiveDate, Utc};
use rusqlite::{params, Connection};
use serde::Serialize;

use super::tasks::{self, Task};
use crate::db::{ids, settings, DbError};

fn key(kind: &str, date: NaiveDate) -> String {
    format!("{kind}:{}", date.format("%Y-%m-%d"))
}

/// Task ids pinned to the top of a day: the must-do first, then the top 3 picks.
pub fn pinned(conn: &Connection, date: NaiveDate) -> Result<Vec<String>, DbError> {
    let mut out = Vec::new();
    if let Some(id) = settings::get(conn, &key("must_do", date))? {
        out.push(id);
    }
    if let Some(json) = settings::get(conn, &key("top3", date))? {
        for id in serde_json::from_str::<Vec<String>>(&json)? {
            if !out.contains(&id) {
                out.push(id);
            }
        }
    }
    Ok(out)
}

pub fn set_must_do(conn: &Connection, date: NaiveDate, task_id: &str) -> Result<(), DbError> {
    tasks::get(conn, task_id)?;
    settings::set(conn, &key("must_do", date), task_id)?;
    settings::set(conn, &key("brief_done", date), &ids::now_iso())
}

/// Saves tomorrow's top 3 (up to 3 tasks) and marks the shutdown done for `today`.
pub fn set_top3(conn: &Connection, today: NaiveDate, task_ids: &[String]) -> Result<(), DbError> {
    if task_ids.len() > 3 {
        return Err(DbError::Invalid("Pick up to 3.".into()));
    }
    for id in task_ids {
        tasks::get(conn, id)?;
    }
    let tomorrow = today + Duration::days(1);
    settings::set(conn, &key("top3", tomorrow), &serde_json::to_string(task_ids)?)?;
    settings::set(conn, &key("shutdown_done", today), &ids::now_iso())
}

pub fn skip_brief(conn: &Connection, date: NaiveDate) -> Result<(), DbError> {
    settings::set(conn, &key("brief_done", date), &ids::now_iso())
}

#[derive(Debug, Serialize)]
pub struct DayFlags {
    pub brief_done: bool,
    pub shutdown_done: bool,
}

pub fn flags(conn: &Connection, date: NaiveDate) -> Result<DayFlags, DbError> {
    Ok(DayFlags {
        brief_done: settings::get(conn, &key("brief_done", date))?.is_some(),
        shutdown_done: settings::get(conn, &key("shutdown_done", date))?.is_some(),
    })
}

#[derive(Debug, Serialize)]
pub struct Shutdown {
    /// Finished today. Shown as wins.
    pub done_today: Vec<Task>,
    /// Due today (or earlier) and still open. Each gets "Tomorrow" or "Later".
    pub left_over: Vec<Task>,
    /// Open tasks to pick tomorrow's top 3 from.
    pub candidates: Vec<Task>,
}

pub fn shutdown(conn: &Connection, now: DateTime<Utc>) -> Result<Shutdown, DbError> {
    let today = ids::local_today(now);
    let today_str = today.format("%Y-%m-%d").to_string();
    // "Finished today" = marked done since local midnight.
    let midnight = crate::scheduler::reminders::local_midnight(today);
    let mut stmt = conn.prepare(
        "SELECT id FROM items WHERE module = 'tasks' AND type = 'task' AND status = 'done' AND updated_at >= ?1 ORDER BY updated_at DESC",
    )?;
    let done_ids: Vec<String> = stmt.query_map(params![ids::to_iso(midnight)], |r| r.get(0))?.collect::<Result<_, _>>()?;
    let done_today = done_ids.iter().map(|id| tasks::get(conn, id)).collect::<Result<Vec<_>, _>>()?;

    let mut open = tasks::list(conn, tasks::TaskView::Active, now)?;
    open.extend(tasks::list(conn, tasks::TaskView::CatchUp, now)?);
    let left_over = open
        .iter()
        .filter(|t| t.data.due_on.as_deref().is_some_and(|d| d <= today_str.as_str()))
        .cloned()
        .collect();
    let mut candidates = open;
    tasks::sort_for_focus(&mut candidates);
    Ok(Shutdown { done_today, left_over, candidates })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_conn;
    use crate::modules::tasks::{create, NewTask};

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn must_do_then_top3_pins() {
        let c = test_conn();
        let a = create(&c, &NewTask { title: "A".into(), ..Default::default() }).unwrap();
        let b = create(&c, &NewTask { title: "B".into(), ..Default::default() }).unwrap();
        set_top3(&c, d("2026-10-05"), &[b.id.clone(), a.id.clone()]).unwrap();
        set_must_do(&c, d("2026-10-06"), &a.id).unwrap();
        assert_eq!(pinned(&c, d("2026-10-06")).unwrap(), vec![a.id.clone(), b.id.clone()]);
        assert!(flags(&c, d("2026-10-05")).unwrap().shutdown_done);
        assert!(flags(&c, d("2026-10-06")).unwrap().brief_done);
        assert!(set_top3(&c, d("2026-10-06"), &[a.id.clone(), a.id.clone(), a.id.clone(), a.id]).is_err());
    }

    #[test]
    fn shutdown_lists_wins_and_leftovers() {
        let c = test_conn();
        let now = Utc::now();
        let today = ids::local_today(now).format("%Y-%m-%d").to_string();
        let done = create(&c, &NewTask { title: "Done".into(), ..Default::default() }).unwrap();
        tasks::set_done(&c, &done.id, true).unwrap();
        create(
            &c,
            &NewTask { title: "Left".into(), data: tasks::TaskData { due_on: Some(today), ..Default::default() }, ..Default::default() },
        )
        .unwrap();
        create(&c, &NewTask { title: "Someday".into(), ..Default::default() }).unwrap();
        let s = shutdown(&c, now).unwrap();
        assert_eq!(s.done_today.len(), 1);
        assert_eq!(s.left_over.iter().map(|t| t.title.as_str()).collect::<Vec<_>>(), vec!["Left"]);
        assert_eq!(s.candidates.len(), 2);
    }
}
