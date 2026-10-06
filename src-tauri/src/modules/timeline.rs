//! The timeline: one log of everything that happened.
//!
//! Spec: logs never contain passwords, tokens, account numbers, or
//! transaction details. Keep `details` to plain, non-sensitive text.

use rusqlite::{params, Connection};

use crate::db::{ids, DbError};

pub fn log(conn: &Connection, item_id: Option<&str>, action: &str, details: Option<&str>) -> Result<(), DbError> {
    conn.execute(
        "INSERT INTO timeline (id, item_id, action, details, at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![ids::new_id(), item_id, action, details, ids::now_iso()],
    )?;
    Ok(())
}
