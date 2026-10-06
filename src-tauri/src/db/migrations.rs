//! Numbered SQL migrations from the top-level `migrations/` folder.
//!
//! Each file is compiled into the app. The applied version is kept in
//! SQLite's `user_version` pragma. Spec hard rule: take a backup before
//! every migration.

use rusqlite::Connection;
use std::path::Path;

use super::DbError;

/// (version, sql). Add new files here in order. Never edit a shipped file.
pub const MIGRATIONS: &[(i64, &str)] = &[
    (1, include_str!("../../../migrations/0001_init.sql")),
    (2, include_str!("../../../migrations/0002_events_archive.sql")),
];

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upgrade_takes_a_backup_first() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("mylife.db");
        let backups = dir.path().join("backups");
        // A database left at version 1 by an older build.
        let mut conn = Connection::open(&db).unwrap();
        conn.execute_batch(MIGRATIONS[0].1).unwrap();
        conn.pragma_update(None, "user_version", 1).unwrap();

        migrate(&mut conn, &db, &backups).unwrap();
        assert_eq!(current_version(&conn).unwrap(), latest_version());
        let names: Vec<_> = std::fs::read_dir(&backups).unwrap().map(|e| e.unwrap().file_name()).collect();
        assert_eq!(names.len(), 1);
        assert!(names[0].to_string_lossy().contains("premigrate-v1"));

        // Running again does nothing and takes no new backup.
        migrate(&mut conn, &db, &backups).unwrap();
        assert_eq!(std::fs::read_dir(&backups).unwrap().count(), 1);
    }
}
