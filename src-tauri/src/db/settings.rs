//! Small key/value settings stored in the encrypted database.

use rusqlite::{params, Connection, OptionalExtension};

use super::DbError;

pub fn get(conn: &Connection, key: &str) -> Result<Option<String>, DbError> {
    Ok(conn.query_row("SELECT value FROM settings WHERE key = ?1", params![key], |r| r.get(0)).optional()?)
}

pub fn set(conn: &Connection, key: &str, value: &str) -> Result<(), DbError> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}
