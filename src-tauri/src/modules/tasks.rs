//! Tasks and projects.
//!
//! A task is a row in `items` with module 'tasks' and type 'task'.
//! Extra fields live in the JSON `data` column (see `TaskData`).
//! Steps are tasks too, joined to their parent through `links`
//! with relation 'step_of'.
//!
//! Missed tasks are never shown as a red overdue list. A task whose due
//! date has passed moves to the "Catch-up" view instead.

use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};

use super::timeline;
use crate::db::{ids, DbError};

pub const STEP_OF: &str = "step_of";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Energy {
    Low,
    Medium,
    High,
}

impl Energy {
    fn as_str(self) -> &'static str {
        match self {
            Energy::Low => "low",
            Energy::Medium => "medium",
            Energy::High => "high",
        }
    }
    fn parse(s: &str) -> Option<Self> {
        match s {
            "low" => Some(Energy::Low),
            "medium" => Some(Energy::Medium),
            "high" => Some(Energy::High),
            _ => None,
        }
    }
}

/// Fields kept in the `data` JSON column.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct TaskData {
    /// Local (Anchorage) due date, `YYYY-MM-DD`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due_on: Option<String>,
    /// Exact due time in UTC, when the user gave one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// True for things saved by quick capture and not sorted yet.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub inbox: bool,
    /// Where quick capture thinks this belongs ('chores', 'gym', 'finance').
    /// Used once those modules exist (Phase 2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_module: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub status: String,
    pub energy: Option<Energy>,
    pub est_minutes: Option<i64>,
    pub actual_minutes: Option<i64>,
    pub data: TaskData,
    /// Set when this task is a step of another task.
    pub parent_id: Option<String>,
    /// Number of steps not yet done.
    pub open_steps: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct NewTask {
    pub title: String,
    pub energy: Option<Energy>,
    pub est_minutes: Option<i64>,
    #[serde(default)]
    pub data: TaskData,
}

/// Partial update. `None` means "leave as is".
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TaskPatch {
    pub title: Option<String>,
    /// `Some(None)` clears the energy tag.
    #[serde(default, deserialize_with = "double_option")]
    pub energy: Option<Option<Energy>>,
    #[serde(default, deserialize_with = "double_option")]
    pub est_minutes: Option<Option<i64>>,
    #[serde(default, deserialize_with = "double_option")]
    pub actual_minutes: Option<Option<i64>>,
    #[serde(default, deserialize_with = "double_option")]
    pub due_on: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub notes: Option<Option<String>>,
    pub inbox: Option<bool>,
}

/// Lets JSON tell apart a missing field (leave) from `null` (clear).
fn double_option<'de, T, D>(de: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: serde::Deserializer<'de>,
{
    Option::<T>::deserialize(de).map(Some)
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskView {
    /// Active tasks that are not in Catch-up. Steps are shown under their parent, not here.
    Active,
    /// Quick capture items waiting to be sorted.
    Inbox,
    /// Active tasks whose due date has passed.
    CatchUp,
    Done,
}

const SELECT_TASK: &str = "
SELECT i.id, i.title, i.status, i.energy, i.est_minutes, i.actual_minutes, i.data,
       i.created_at, i.updated_at,
       (SELECT l.to_id FROM links l WHERE l.from_id = i.id AND l.relation = 'step_of') AS parent_id,
       (SELECT count(*) FROM links l JOIN items s ON s.id = l.from_id
         WHERE l.to_id = i.id AND l.relation = 'step_of' AND s.status = 'active') AS open_steps
FROM items i
WHERE i.module = 'tasks' AND i.type = 'task'";

fn row_to_task(r: &Row) -> rusqlite::Result<Task> {
    let data_json: String = r.get(6)?;
    let energy: Option<String> = r.get(3)?;
    Ok(Task {
        id: r.get(0)?,
        title: r.get(1)?,
        status: r.get(2)?,
        energy: energy.as_deref().and_then(Energy::parse),
        est_minutes: r.get(4)?,
        actual_minutes: r.get(5)?,
        // Bad JSON should never happen, but must not crash the list.
        data: serde_json::from_str(&data_json).unwrap_or_default(),
        created_at: r.get(7)?,
        updated_at: r.get(8)?,
        parent_id: r.get(9)?,
        open_steps: r.get(10)?,
    })
}

fn validate(title: &str, est: Option<i64>, due_on: Option<&str>) -> Result<(), DbError> {
    if title.trim().is_empty() {
        return Err(DbError::Invalid("Title can't be empty.".into()));
    }
    if matches!(est, Some(m) if m <= 0) {
        return Err(DbError::Invalid("Time estimate must be more than 0 minutes.".into()));
    }
    if let Some(d) = due_on {
        NaiveDate::parse_from_str(d, "%Y-%m-%d").map_err(|_| DbError::Invalid("Due date must be YYYY-MM-DD.".into()))?;
    }
    Ok(())
}

pub fn create(conn: &Connection, t: &NewTask) -> Result<Task, DbError> {
    validate(&t.title, t.est_minutes, t.data.due_on.as_deref())?;
    let id = ids::new_id();
    let now = ids::now_iso();
    conn.execute(
        "INSERT INTO items (id, module, type, title, energy, est_minutes, data, created_at, updated_at)
         VALUES (?1, 'tasks', 'task', ?2, ?3, ?4, ?5, ?6, ?6)",
        params![
            id,
            t.title.trim(),
            t.energy.map(Energy::as_str),
            t.est_minutes,
            serde_json::to_string(&t.data)?,
            now
        ],
    )?;
    timeline::log(conn, Some(&id), "task_created", None)?;
    get(conn, &id)
}

pub fn get(conn: &Connection, id: &str) -> Result<Task, DbError> {
    conn.query_row(&format!("{SELECT_TASK} AND i.id = ?1"), params![id], row_to_task)
        .optional()?
        .ok_or(DbError::NotFound)
}

pub fn update(conn: &Connection, id: &str, p: &TaskPatch) -> Result<Task, DbError> {
    let mut t = get(conn, id)?;
    if let Some(title) = &p.title {
        t.title = title.trim().to_string();
    }
    if let Some(e) = p.energy {
        t.energy = e;
    }
    if let Some(m) = p.est_minutes {
        t.est_minutes = m;
    }
    if let Some(m) = p.actual_minutes {
        t.actual_minutes = m;
    }
    if let Some(d) = &p.due_on {
        t.data.due_on = d.clone();
        // A new date replaces any exact time from quick capture.
        t.data.due_at = None;
    }
    if let Some(n) = &p.notes {
        t.data.notes = n.clone();
    }
    if let Some(b) = p.inbox {
        t.data.inbox = b;
    }
    validate(&t.title, t.est_minutes, t.data.due_on.as_deref())?;
    conn.execute(
        "UPDATE items SET title = ?2, energy = ?3, est_minutes = ?4, actual_minutes = ?5, data = ?6, updated_at = ?7
         WHERE id = ?1",
        params![
            id,
            t.title,
            t.energy.map(Energy::as_str),
            t.est_minutes,
            t.actual_minutes,
            serde_json::to_string(&t.data)?,
            ids::now_iso()
        ],
    )?;
    timeline::log(conn, Some(id), "task_updated", None)?;
    get(conn, id)
}

/// Marks a task done (or back to active). Completing also clears the inbox flag.
pub fn set_done(conn: &Connection, id: &str, done: bool) -> Result<Task, DbError> {
    let mut t = get(conn, id)?;
    if t.status == "archived" {
        return Err(DbError::Invalid("This task is archived.".into()));
    }
    if done {
        t.data.inbox = false;
    }
    conn.execute(
        "UPDATE items SET status = ?2, data = ?3, updated_at = ?4 WHERE id = ?1",
        params![id, if done { "done" } else { "active" }, serde_json::to_string(&t.data)?, ids::now_iso()],
    )?;
    timeline::log(conn, Some(id), if done { "task_done" } else { "task_reopened" }, None)?;
    get(conn, id)
}

/// Soft delete. Spec: deletes set `archived_at`. Nothing is removed.
pub fn archive(conn: &Connection, id: &str) -> Result<(), DbError> {
    get(conn, id)?;
    let now = ids::now_iso();
    conn.execute(
        "UPDATE items SET status = 'archived', archived_at = ?2, updated_at = ?2 WHERE id = ?1",
        params![id, now],
    )?;
    timeline::log(conn, Some(id), "task_archived", None)?;
    Ok(())
}

/// Splits a task into steps, added after any steps it already has.
/// Each step is a small task of its own, linked to the parent.
pub fn add_steps(conn: &Connection, parent_id: &str, titles: &[String]) -> Result<Vec<Task>, DbError> {
    let parent = get(conn, parent_id)?;
    if parent.parent_id.is_some() {
        return Err(DbError::Invalid("Steps can't have their own steps.".into()));
    }
    let titles: Vec<&str> = titles.iter().map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
    if titles.is_empty() {
        return Err(DbError::Invalid("Add at least one step.".into()));
    }
    let mut out = Vec::new();
    for title in titles {
        let step = create(
            conn,
            &NewTask {
                title: title.to_string(),
                energy: parent.energy,
                ..Default::default()
            },
        )?;
        conn.execute(
            "INSERT INTO links (from_id, to_id, relation) VALUES (?1, ?2, ?3)",
            params![step.id, parent_id, STEP_OF],
        )?;
        out.push(get(conn, &step.id)?);
    }
    Ok(out)
}

/// Steps of a task, in the order they were added.
pub fn steps(conn: &Connection, parent_id: &str) -> Result<Vec<Task>, DbError> {
    let sql = format!(
        "{SELECT_TASK} AND i.status != 'archived' AND i.id IN
           (SELECT from_id FROM links WHERE to_id = ?1 AND relation = 'step_of')
         ORDER BY i.id"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![parent_id], row_to_task)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// True when the task's due date is before today (Anchorage time).
pub fn is_catch_up(t: &Task, today: NaiveDate) -> bool {
    t.status == "active"
        && t.data
            .due_on
            .as_deref()
            .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
            .is_some_and(|d| d < today)
}

/// Lists top-level tasks for one view. `now` decides what "today" is.
pub fn list(conn: &Connection, view: TaskView, now: DateTime<Utc>) -> Result<Vec<Task>, DbError> {
    let today = ids::local_today(now);
    let status = if view == TaskView::Done { "done" } else { "active" };
    let sql = format!(
        "{SELECT_TASK} AND i.status = ?1
           AND NOT EXISTS (SELECT 1 FROM links l WHERE l.from_id = i.id AND l.relation = 'step_of')
         ORDER BY i.id"
    );
    let mut stmt = conn.prepare(&sql)?;
    let all: Vec<Task> = stmt.query_map(params![status], row_to_task)?.collect::<Result<_, _>>()?;
    let mut out: Vec<Task> = match view {
        TaskView::Active => all.into_iter().filter(|t| !is_catch_up(t, today)).collect(),
        TaskView::Inbox => all.into_iter().filter(|t| t.data.inbox).collect(),
        TaskView::CatchUp => all.into_iter().filter(|t| is_catch_up(t, today)).collect(),
        TaskView::Done => all,
    };
    if view == TaskView::Done {
        // Most recently finished first.
        out.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    } else {
        sort_for_focus(&mut out);
    }
    Ok(out)
}

/// Order used for "what next": due date first (soonest), undated after, then oldest first.
pub fn sort_for_focus(tasks: &mut [Task]) {
    tasks.sort_by(|a, b| {
        let da = a.data.due_on.as_deref();
        let db = b.data.due_on.as_deref();
        match (da, db) {
            (Some(x), Some(y)) => x.cmp(y),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
        .then_with(|| a.id.cmp(&b.id))
    });
}

/// Moves a Catch-up task to a new day (default: today). This is the
/// "pick 1 to clear" action: no penalty, just a fresh date.
pub fn reschedule(conn: &Connection, id: &str, to: NaiveDate) -> Result<Task, DbError> {
    update(
        conn,
        id,
        &TaskPatch {
            due_on: Some(Some(to.format("%Y-%m-%d").to_string())),
            ..Default::default()
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_conn;
    use chrono::TimeZone;

    fn new(title: &str, due: Option<&str>) -> NewTask {
        NewTask {
            title: title.into(),
            data: TaskData { due_on: due.map(Into::into), ..Default::default() },
            ..Default::default()
        }
    }

    // Oct 6 2026, 20:00 UTC = 12:00 in Anchorage.
    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 6, 20, 0, 0).unwrap()
    }

    #[test]
    fn create_edit_complete() {
        let c = test_conn();
        let t = create(
            &c,
            &NewTask { title: "  Call dentist ".into(), energy: Some(Energy::Low), est_minutes: Some(5), ..Default::default() },
        )
        .unwrap();
        assert_eq!(t.title, "Call dentist");
        assert_eq!(t.energy, Some(Energy::Low));

        let t = update(
            &c,
            &t.id,
            &TaskPatch { energy: Some(None), est_minutes: Some(Some(10)), due_on: Some(Some("2026-10-07".into())), ..Default::default() },
        )
        .unwrap();
        assert_eq!(t.energy, None);
        assert_eq!(t.est_minutes, Some(10));
        assert_eq!(t.data.due_on.as_deref(), Some("2026-10-07"));

        let t = set_done(&c, &t.id, true).unwrap();
        assert_eq!(t.status, "done");
        assert_eq!(list(&c, TaskView::Done, now()).unwrap().len(), 1);
        assert!(list(&c, TaskView::Active, now()).unwrap().is_empty());
    }

    #[test]
    fn rejects_bad_input() {
        let c = test_conn();
        assert!(create(&c, &new("  ", None)).is_err());
        assert!(create(&c, &new("x", Some("10/07/2026"))).is_err());
        assert!(create(&c, &NewTask { title: "x".into(), est_minutes: Some(0), ..Default::default() }).is_err());
    }

    #[test]
    fn missed_tasks_go_to_catch_up_not_overdue() {
        let c = test_conn();
        create(&c, &new("Yesterday", Some("2026-10-05"))).unwrap();
        create(&c, &new("Today", Some("2026-10-06"))).unwrap();
        create(&c, &new("Someday", None)).unwrap();
        let active: Vec<_> = list(&c, TaskView::Active, now()).unwrap().into_iter().map(|t| t.title).collect();
        assert_eq!(active, vec!["Today", "Someday"]);
        let catch_up = list(&c, TaskView::CatchUp, now()).unwrap();
        assert_eq!(catch_up.len(), 1);
        assert_eq!(catch_up[0].title, "Yesterday");

        // Clearing it moves it back without a penalty.
        reschedule(&c, &catch_up[0].id, ids::local_today(now())).unwrap();
        assert!(list(&c, TaskView::CatchUp, now()).unwrap().is_empty());
    }

    #[test]
    fn catch_up_uses_anchorage_date() {
        let c = test_conn();
        create(&c, &new("Due Oct 5", Some("2026-10-05"))).unwrap();
        // 07:00 UTC Oct 6 is still Oct 5 in Anchorage, so not missed yet.
        let early = Utc.with_ymd_and_hms(2026, 10, 6, 7, 0, 0).unwrap();
        assert!(list(&c, TaskView::CatchUp, early).unwrap().is_empty());
    }

    #[test]
    fn split_into_steps() {
        let c = test_conn();
        let p = create(&c, &NewTask { title: "Clean garage".into(), energy: Some(Energy::High), ..Default::default() }).unwrap();
        let s = add_steps(&c, &p.id, &["Grab trash bags".into(), "".into(), "Clear one shelf".into()]).unwrap();
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].parent_id.as_deref(), Some(p.id.as_str()));
        assert_eq!(s[0].energy, Some(Energy::High));
        assert_eq!(get(&c, &p.id).unwrap().open_steps, 2);

        set_done(&c, &s[0].id, true).unwrap();
        assert_eq!(get(&c, &p.id).unwrap().open_steps, 1);

        // Steps show under the parent, not in the main list.
        let top: Vec<_> = list(&c, TaskView::Active, now()).unwrap().into_iter().map(|t| t.title).collect();
        assert_eq!(top, vec!["Clean garage"]);
        assert_eq!(steps(&c, &p.id).unwrap().len(), 2);

        // No steps of steps.
        assert!(add_steps(&c, &s[0].id, &["x".into()]).is_err());
    }

    #[test]
    fn inbox_and_archive() {
        let c = test_conn();
        let t = create(
            &c,
            &NewTask { title: "oil change".into(), data: TaskData { inbox: true, ..Default::default() }, ..Default::default() },
        )
        .unwrap();
        assert_eq!(list(&c, TaskView::Inbox, now()).unwrap().len(), 1);
        archive(&c, &t.id).unwrap();
        assert!(list(&c, TaskView::Inbox, now()).unwrap().is_empty());
        // Soft delete: the row is still there.
        assert_eq!(get(&c, &t.id).unwrap().status, "archived");
        assert!(set_done(&c, &t.id, true).is_err());
    }

    #[test]
    fn patch_json_tells_missing_from_null() {
        let p: TaskPatch = serde_json::from_str(r#"{"energy": null}"#).unwrap();
        assert_eq!(p.energy, Some(None));
        let p: TaskPatch = serde_json::from_str(r#"{}"#).unwrap();
        assert_eq!(p.energy, None);
    }
}
