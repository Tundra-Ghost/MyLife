//! Trigger checks and the action runner. The agent calls `run` every 60 seconds.
//!
//! Supported in Phase 1: `time` triggers (one-off `at` or repeating `rrule`)
//! and `offset` triggers on task due dates. Actions: `notify` and
//! `create_task`. Other trigger and action types are saved but not run yet.

use chrono::{DateTime, NaiveDate, Timelike, Utc};
use rusqlite::Connection;

use super::store::{self, SavedRule};
use super::{offset_fire, Action, Trigger};
use crate::db::{ids, DbError};
use crate::modules::tasks::{self, NewTask, TaskView};
use crate::scheduler::{self, reminders};

/// The only date field offset rules can read so far.
pub const TASK_DUE_ON: &str = "task.due_on";

/// Runs every enabled rule. Each rule remembers when it last fired, so a
/// long sleep still fires each rule only once.
pub fn run(conn: &Connection, now: DateTime<Utc>) -> Result<(), DbError> {
    for rule in store::list(conn)?.into_iter().filter(|r| r.enabled) {
        match &rule.rule.trigger {
            Trigger::Time { .. } => run_time(conn, &rule, now)?,
            Trigger::Offset { date_field, days_before, .. } if date_field.as_deref() == Some(TASK_DUE_ON) => {
                sync_task_offsets(conn, &rule, *days_before, now)?
            }
            _ => {}
        }
    }
    sync_exact_task_times(conn, now)?;
    Ok(())
}

fn parse(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s).ok().map(|d| d.with_timezone(&Utc))
}

/// A time rule fires at most once per check, even after a long sleep.
fn run_time(conn: &Connection, rule: &SavedRule, now: DateTime<Utc>) -> Result<(), DbError> {
    let Trigger::Time { at, rrule } = &rule.rule.trigger else { return Ok(()) };
    let created = parse(&rule.created_at).unwrap_or(now);
    // Never look back past the last time it fired. For a new rule, look back one
    // minute, so a reminder set for "now" still fires.
    let since = rule.last_fired_at.as_deref().and_then(parse).unwrap_or(created - chrono::Duration::minutes(1));
    let at = at.as_deref().and_then(parse);
    let fire = match rrule {
        Some(r) => {
            // Repeats start from `at` if given, else from when the rule was made (to the minute).
            let start = at.unwrap_or_else(|| created.with_second(0).unwrap());
            scheduler::missed_once(r, start, since, now).map_err(DbError::Invalid)?
        }
        None => at.filter(|t| *t > since && *t <= now),
    };
    if let Some(fire) = fire {
        run_actions(conn, rule, None, &rule.rule.name, fire)?;
        store::mark_fired(conn, &rule.id, &ids::to_iso(now))?;
    }
    Ok(())
}

/// Keeps one reminder per task for an offset rule, following due date changes.
fn sync_task_offsets(conn: &Connection, rule: &SavedRule, days_before: i64, now: DateTime<Utc>) -> Result<(), DbError> {
    for task in tasks::list(conn, TaskView::Active, now)? {
        let Some(due) = task.data.due_on.as_deref().and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()) else {
            continue;
        };
        // A task with an exact time already reminds at that time.
        if days_before == 0 && task.data.due_at.is_some() {
            continue;
        }
        let fire = offset_fire(due, days_before);
        // Don't remind about a task that was added after its reminder time.
        if parse(&task.created_at).is_some_and(|c| fire < c) {
            continue;
        }
        let title = format!("Due: {}", task.title);
        match reminders::open_for(conn, Some(&rule.id), &task.id)? {
            Some(r) if r.fire_at != ids::to_iso(fire) && r.state == "pending" => reminders::reschedule(conn, &r.id, fire, &title)?,
            Some(_) => {}
            // Don't make new reminders for dates that already passed (those tasks are in Catch-up).
            None if ids::local_today(now) <= due => {
                run_actions(conn, rule, Some(&task.id), &title, fire)?;
            }
            None => {}
        }
    }
    close_finished(conn)
}

/// Quick capture times ("dentist at 3pm") remind at that exact time.
fn sync_exact_task_times(conn: &Connection, now: DateTime<Utc>) -> Result<(), DbError> {
    for task in tasks::list(conn, TaskView::Active, now)? {
        let Some(at) = task.data.due_at.as_deref().and_then(parse) else { continue };
        if reminders::open_for(conn, None, &task.id)?.is_none() && at > now - chrono::Duration::days(1) {
            reminders::create(
                conn,
                &reminders::NewReminder { rule_id: None, item_id: Some(&task.id), title: &task.title, fire_at: at, max_ladder: 2, urgent: false },
            )?;
        }
    }
    Ok(())
}

/// Tasks that were finished or archived stop reminding.
fn close_finished(conn: &Connection) -> Result<(), DbError> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT r.item_id FROM reminders r JOIN items i ON i.id = r.item_id
         WHERE r.state IN ('pending','fired','snoozed') AND i.status != 'active'",
    )?;
    let ids: Vec<String> = stmt.query_map([], |r| r.get(0))?.collect::<Result<_, _>>()?;
    for id in ids {
        reminders::close_for_item(conn, &id)?;
    }
    Ok(())
}

fn run_actions(conn: &Connection, rule: &SavedRule, item_id: Option<&str>, title: &str, fire: DateTime<Utc>) -> Result<(), DbError> {
    for action in &rule.rule.actions {
        match action {
            Action::Notify(fields) => {
                let text = fields.get("text").and_then(|v| v.as_str()).unwrap_or(title);
                let urgent = fields.get("urgent").and_then(|v| v.as_bool()).unwrap_or(false);
                reminders::create(
                    conn,
                    &reminders::NewReminder {
                        rule_id: Some(&rule.id),
                        item_id,
                        title: text,
                        fire_at: fire,
                        max_ladder: rule.rule.max_ladder,
                        urgent,
                    },
                )?;
            }
            Action::CreateTask(fields) => {
                let t = fields.get("title").and_then(|v| v.as_str()).unwrap_or(title);
                tasks::create(
                    conn,
                    &NewTask {
                        title: t.to_string(),
                        est_minutes: fields.get("est_minutes").and_then(|v| v.as_i64()),
                        energy: fields.get("energy").and_then(|v| serde_json::from_value(v.clone()).ok()),
                        data: tasks::TaskData {
                            due_on: Some(ids::local_today(fire).format("%Y-%m-%d").to_string()),
                            ..Default::default()
                        },
                    },
                )?;
            }
            // Other actions arrive with the modules that need them.
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::ids::USER_TZ;
    use crate::db::test_conn;
    use crate::rules::parse as parse_rule;
    use crate::scheduler::reminders::{active, deliver};
    use chrono::TimeZone;

    fn ak(d: u32, h: u32, m: u32) -> DateTime<Utc> {
        USER_TZ.with_ymd_and_hms(2026, 10, d, h, m, 0).earliest().unwrap().with_timezone(&Utc)
    }

    fn add_rule(c: &Connection, json: &str, created: DateTime<Utc>) -> SavedRule {
        let s = store::create(c, "tasks", &parse_rule(json).unwrap()).unwrap();
        c.execute("UPDATE rules SET created_at = ?2 WHERE id = ?1", rusqlite::params![s.id, ids::to_iso(created)]).unwrap();
        store::get(c, &s.id).unwrap()
    }

    #[test]
    fn repeating_time_rule_fires_within_a_tick() {
        let c = test_conn();
        let at = ids::to_iso(ak(5, 8, 0));
        add_rule(
            &c,
            &format!(r#"{{"name":"Meds","trigger":{{"type":"time","at":"{at}","rrule":"FREQ=DAILY"}},"actions":[{{"type":"notify"}}]}}"#),
            ak(6, 7, 0),
        );
        run(&c, ak(6, 7, 59)).unwrap();
        assert!(active(&c, ak(6, 8, 0)).unwrap().is_empty());
        // The tick right after 8:00 makes the reminder and it notifies.
        run(&c, ak(6, 8, 0)).unwrap();
        assert_eq!(deliver(&c, ak(6, 8, 0)).unwrap().len(), 1);
        // Next tick: nothing new.
        run(&c, ak(6, 8, 1)).unwrap();
        assert_eq!(active(&c, ak(6, 8, 1)).unwrap().len(), 1);
    }

    #[test]
    fn missed_repeats_fire_once_after_sleep() {
        let c = test_conn();
        let at = ids::to_iso(ak(1, 8, 0));
        add_rule(
            &c,
            &format!(r#"{{"name":"Meds","trigger":{{"type":"time","at":"{at}","rrule":"FREQ=DAILY"}},"actions":[{{"type":"notify"}}]}}"#),
            ak(1, 7, 0),
        );
        run(&c, ak(1, 7, 1)).unwrap();
        // Asleep for 4 days.
        run(&c, ak(5, 12, 0)).unwrap();
        assert_eq!(active(&c, ak(5, 12, 0)).unwrap().len(), 1);
    }

    #[test]
    fn one_off_rule_and_create_task() {
        let c = test_conn();
        let at = ids::to_iso(ak(6, 10, 0));
        add_rule(
            &c,
            &format!(r#"{{"name":"Renew tabs","trigger":{{"type":"time","at":"{at}"}},"actions":[{{"type":"create_task","title":"Renew car tabs","est_minutes":20}}]}}"#),
            ak(6, 9, 0),
        );
        run(&c, ak(6, 10, 0)).unwrap();
        run(&c, ak(6, 10, 1)).unwrap();
        let t = tasks::list(&c, TaskView::Active, ak(6, 10, 1)).unwrap();
        assert_eq!(t.len(), 1);
        assert_eq!(t[0].title, "Renew car tabs");
        assert_eq!(t[0].data.due_on.as_deref(), Some("2026-10-06"));
    }

    /// Pins every item's creation time so tests don't depend on today's date.
    fn backdate_items(c: &Connection, to: DateTime<Utc>) {
        c.execute("UPDATE items SET created_at = ?1", rusqlite::params![ids::to_iso(to)]).unwrap();
    }

    #[test]
    fn task_due_today_starter() {
        let c = test_conn();
        store::seed_starters(&c).unwrap();
        let t = tasks::create(
            &c,
            &NewTask { title: "Call bank".into(), data: tasks::TaskData { due_on: Some("2026-10-07".into()), ..Default::default() }, ..Default::default() },
        )
        .unwrap();
        run(&c, ak(6, 12, 1)).unwrap();
        // Fires at 9 AM on the due date.
        assert!(deliver(&c, ak(7, 8, 59)).unwrap().is_empty());
        let n = deliver(&c, ak(7, 9, 0)).unwrap();
        assert_eq!(n[0].title, "Due: Call bank");

        // Moving the due date moves an untouched reminder.
        let t2 = tasks::create(
            &c,
            &NewTask { title: "Email".into(), data: tasks::TaskData { due_on: Some("2026-10-08".into()), ..Default::default() }, ..Default::default() },
        )
        .unwrap();
        run(&c, ak(6, 12, 2)).unwrap();
        tasks::update(&c, &t2.id, &tasks::TaskPatch { due_on: Some(Some("2026-10-09".into())), ..Default::default() }).unwrap();
        run(&c, ak(6, 12, 3)).unwrap();
        assert!(deliver(&c, ak(8, 9, 0)).unwrap().is_empty());
        assert_eq!(deliver(&c, ak(9, 9, 0)).unwrap().len(), 1);

        // Finishing a task stops its reminder.
        tasks::set_done(&c, &t.id, true).unwrap();
        run(&c, ak(9, 9, 2)).unwrap();
        assert!(active(&c, ak(9, 9, 2)).unwrap().iter().all(|r| r.item_id.as_deref() != Some(t.id.as_str())));
    }

    #[test]
    fn quick_capture_time_reminds_at_that_time() {
        let c = test_conn();
        tasks::create(
            &c,
            &NewTask {
                title: "Dentist".into(),
                data: tasks::TaskData { due_on: Some("2026-10-06".into()), due_at: Some(ids::to_iso(ak(6, 15, 0))), ..Default::default() },
                ..Default::default()
            },
        )
        .unwrap();
        backdate_items(&c, ak(6, 0, 0));
        store::seed_starters(&c).unwrap();
        run(&c, ak(6, 12, 1)).unwrap();
        run(&c, ak(6, 12, 2)).unwrap();
        // One reminder at 3 PM, not a second "due today" one.
        assert_eq!(deliver(&c, ak(6, 15, 0)).unwrap().len(), 1);
        assert_eq!(active(&c, ak(6, 15, 0)).unwrap().len(), 1);
    }

    #[test]
    fn no_reminder_for_a_time_before_the_task_existed() {
        let c = test_conn();
        store::seed_starters(&c).unwrap();
        tasks::create(
            &c,
            &NewTask { title: "Late add".into(), data: tasks::TaskData { due_on: Some("2026-10-06".into()), ..Default::default() }, ..Default::default() },
        )
        .unwrap();
        // Added at 8 PM, after the 9 AM "due today" time.
        backdate_items(&c, ak(6, 20, 0));
        run(&c, ak(6, 20, 1)).unwrap();
        assert!(active(&c, ak(6, 20, 1)).unwrap().is_empty());
    }

    #[test]
    fn disabled_rules_do_nothing() {
        let c = test_conn();
        let at = ids::to_iso(ak(6, 10, 0));
        let r = add_rule(
            &c,
            &format!(r#"{{"name":"x","trigger":{{"type":"time","at":"{at}"}},"actions":[{{"type":"notify"}}]}}"#),
            ak(6, 9, 0),
        );
        store::set_enabled(&c, &r.id, false).unwrap();
        run(&c, ak(6, 10, 0)).unwrap();
        assert!(active(&c, ak(6, 10, 0)).unwrap().is_empty());
    }
}
