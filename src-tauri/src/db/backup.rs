//! File backups of the encrypted database.
//!
//! Backups are byte copies of `mylife.db`, so they stay encrypted with the
//! same app password. The scheduled daily backup and restore screen come in
//! a later Phase 1 step. For now this is used before migrations.

use rusqlite::Connection;
use std::path::{Path, PathBuf};

use super::DbError;

/// Copies the database file into `backup_dir` with a timestamped name.
/// Returns the path of the new backup.
pub fn copy_backup(
    conn: &Connection,
    db_path: &Path,
    backup_dir: &Path,
    label: &str,
) -> Result<PathBuf, DbError> {
    // Flush the write-ahead log so the main file holds everything.
    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
    std::fs::create_dir_all(backup_dir)?;
    let stamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ");
    let dest = backup_dir.join(format!("mylife-{stamp}-{label}.db"));
    std::fs::copy(db_path, &dest)?;
    Ok(dest)
}
