//! Numbered SQL migrations from the top-level `migrations/` folder.
//!
//! Each file is compiled into the app. The applied version is kept in
//! SQLite's `user_version` pragma. Spec hard rule: take a backup before
//! every migration.

use rusqlite::Connection;
use std::path::Path;

use super::DbError;

/// (version, sql). Add new files here in order. Never edit a shipped file.
pub const MIGRATIONS: &[(i64, &str)] = &[(1, include_str!("../../../migrations/0001_init.sql"))];

/// The newest schema version this build knows.
pub fn latest_version() -> i64 {
    MIGRATIONS.last().map(|m| m.0).unwrap_or(0)
}

pub fn current_version(conn: &Connection) -> Result<i64, DbError> {
    Ok(conn.query_row("PRAGMA user_version", [], |r| r.get(0))?)
}

/// Applies any migrations newer than the current version.
///
/// `db_path` and `backup_dir` are used to copy the (still encrypted) file
/// before anything changes. A brand new, empty database is not backed up
/// because there is nothing to lose.
pub fn migrate(conn: &mut Connection, db_path: &Path, backup_dir: &Path) -> Result<(), DbError> {
    let from = current_version(conn)?;
    if from >= latest_version() {
        return Ok(());
    }
    if from > 0 {
        super::backup::copy_backup(conn, db_path, backup_dir, &format!("premigrate-v{from}"))?;
    }
    for (version, sql) in MIGRATIONS.iter().filter(|m| m.0 > from) {
        // One transaction per migration so a failure leaves the old schema intact.
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(None, "user_version", version)?;
        tx.commit()?;
    }
    Ok(())
}
