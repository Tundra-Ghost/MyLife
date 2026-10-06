//! Saved rules in the `rules` table, plus each module's starter rules.

use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::Serialize;

use super::{parse, Rule};
use crate::db::{ids, DbError};

/// Starter rules ship in each module's folder (spec repo layout).
const STARTERS: &[(&str, &str)] = &[("tasks", include_str!("../../../src/modules/tasks/starter-rules.json"))];

#[derive(Debug, Clone, Serialize)]
pub struct SavedRule {
    pub id: String,
    pub module: String,
    pub enabled: bool,
    pub rule: Rule,
    pub last_fired_at: Option<String>,
    pub created_at: String,
}

const SELECT: &str = "SELECT id, module, name, enabled, trigger_json, conditions_json, actions_json, max_ladder, last_fired_at, created_at FROM rules";

fn row(r: &Row) -> rusqlite::Result<(String, String, bool, String, Option<String>, String)> {
    let name: String = r.get(2)?;
    let json = format!(
        r#"{{"name":{},"trigger":{},"conditions":{},"actions":{},"max_ladder":{}}}"#,
        serde_json::to_string(&name).unwrap(),
        r.get::<_, String>(4)?,
        r.get::<_, String>(5)?,
        r.get::<_, String>(6)?,
        r.get::<_, i64>(7)?
    );
    Ok((r.get(0)?, r.get(1)?, r.get::<_, i64>(3)? != 0, json, r.get(8)?, r.get(9)?))
}

fn to_saved(t: (String, String, bool, String, Option<String>, String)) -> Result<SavedRule, DbError> {
    let (id, module, enabled, json, last_fired_at, created_at) = t;
    Ok(SavedRule { id, module, enabled, rule: parse(&json).map_err(DbError::Invalid)?, last_fired_at, created_at })
}

pub fn list(conn: &Connection) -> Result<Vec<SavedRule>, DbError> {
    let mut stmt = conn.prepare(&format!("{SELECT} ORDER BY module, name"))?;
    let rows: Vec<_> = stmt.query_map([], row)?.collect::<Result<_, _>>()?;
    rows.into_iter().map(to_saved).collect()
}

pub fn get(conn: &Connection, id: &str) -> Result<SavedRule, DbError> {
    let t = conn.query_row(&format!("{SELECT} WHERE id = ?1"), params![id], row).optional()?.ok_or(DbError::NotFound)?;
    to_saved(t)
}

pub fn create(conn: &Connection, module: &str, rule: &Rule) -> Result<SavedRule, DbError> {
    // Round-trip through the parser so saved rules are always valid.
    let rule = parse(&serde_json::to_string(rule)?).map_err(DbError::Invalid)?;
    if rule.name.trim().is_empty() {
        return Err(DbError::Invalid("Give the rule a name.".into()));
    }
    let id = ids::new_id();
    let now = ids::now_iso();
    conn.execute(
        "INSERT INTO rules (id, module, name, enabled, trigger_json, conditions_json, actions_json, max_ladder, created_at, updated_at)
         VALUES (?1, ?2, ?3, 1, ?4, ?5, ?6, ?7, ?8, ?8)",
        params![
            id,
            module,
            rule.name.trim(),
            serde_json::to_string(&rule.trigger)?,
            serde_json::to_string(&rule.conditions)?,
            serde_json::to_string(&rule.actions)?,
            rule.max_ladder,
            now
        ],
    )?;
    get(conn, &id)
}

pub fn set_enabled(conn: &Connection, id: &str, enabled: bool) -> Result<SavedRule, DbError> {
    get(conn, id)?;
    conn.execute(
        "UPDATE rules SET enabled = ?2, updated_at = ?3 WHERE id = ?1",
        params![id, enabled as i64, ids::now_iso()],
    )?;
    get(conn, id)
}

/// Deletes a rule. The UI asks first. Reminders it made are kept.
pub fn delete(conn: &Connection, id: &str) -> Result<(), DbError> {
    get(conn, id)?;
    conn.execute("UPDATE reminders SET rule_id = NULL WHERE rule_id = ?1", params![id])?;
    conn.execute("DELETE FROM rules WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn mark_fired(conn: &Connection, id: &str, at: &str) -> Result<(), DbError> {
    conn.execute("UPDATE rules SET last_fired_at = ?2 WHERE id = ?1", params![id, at])?;
    Ok(())
}

/// Adds any starter rule not already saved (matched by module and name).
/// Runs once per module, so a starter the user deleted stays deleted.
pub fn seed_starters(conn: &Connection) -> Result<(), DbError> {
    for (module, json) in STARTERS {
        let key = format!("starters_seeded:{module}");
        if crate::db::settings::get(conn, &key)?.is_some() {
            continue;
        }
        let rules: Vec<Rule> = serde_json::from_str(json)?;
        for rule in &rules {
            create(conn, module, rule)?;
        }
        crate::db::settings::set(conn, &key, &ids::now_iso())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_conn;

    #[test]
    fn starters_seed_once() {
        let c = test_conn();
        seed_starters(&c).unwrap();
        seed_starters(&c).unwrap();
        let all = list(&c).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].rule.name, "Task due today");
        // A deleted starter stays deleted.
        delete(&c, &all[0].id).unwrap();
        seed_starters(&c).unwrap();
        assert!(list(&c).unwrap().is_empty());
    }

    #[test]
    fn create_toggle_roundtrip() {
        let c = test_conn();
        let rule = parse(r#"{"name":"Water","trigger":{"type":"time","rrule":"FREQ=DAILY"},"actions":[{"type":"notify"}],"max_ladder":1}"#).unwrap();
        let s = create(&c, "tasks", &rule).unwrap();
        assert!(s.enabled);
        assert_eq!(s.rule, rule);
        assert!(!set_enabled(&c, &s.id, false).unwrap().enabled);
    }
}
