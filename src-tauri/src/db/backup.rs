//! File backups of the encrypted database, and restore.
//!
//! Backups are byte copies of `mylife.db`, so they stay encrypted with the
//! app password. A daily backup goes to the folder the user picks. Restore
//! always backs up the current file first (spec build rule 6).

use chrono::{DateTime, Duration, Utc};
use rusqlite::Connection;
use serde::Serialize;
use std::path::{Path, PathBuf};

use super::{settings, DbError, DbPaths};

/// Daily backups to keep. Older daily ones are removed. Other backups stay.
pub const KEEP_DAILY: usize = 30;
const DIR_KEY: &str = "backup_dir";
const LAST_KEY: &str = "last_backup_at";
const PREFIX: &str = "mylife-";

/// Copies the database file into `backup_dir` with a timestamped name.
/// Returns the path of the new backup.
pub fn copy_backup(conn: &Connection, db_path: &Path, backup_dir: &Path, label: &str) -> Result<PathBuf, DbError> {
    // Flush the write-ahead log so the main file holds everything.
    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
    std::fs::create_dir_all(backup_dir)?;
    let stamp = Utc::now().format("%Y%m%dT%H%M%SZ");
    let dest = backup_dir.join(format!("{PREFIX}{stamp}-{label}.db"));
    std::fs::copy(db_path, &dest)?;
    Ok(dest)
}

/// The folder daily backups go to: the user's pick, or the default.
pub fn backup_dir(conn: &Connection, paths: &DbPaths) -> Result<PathBuf, DbError> {
    Ok(settings::get(conn, DIR_KEY)?.map(PathBuf::from).unwrap_or_else(|| paths.backup_dir.clone()))
}

pub fn set_backup_dir(conn: &Connection, dir: &Path) -> Result<(), DbError> {
    std::fs::create_dir_all(dir)?;
    // Make sure we can write there before saving the choice.
    let probe = dir.join(".mylife-write-test");
    std::fs::write(&probe, b"ok")?;
    std::fs::remove_file(&probe)?;
    settings::set(conn, DIR_KEY, &dir.to_string_lossy())
}

pub fn last_backup_at(conn: &Connection) -> Result<Option<String>, DbError> {
    settings::get(conn, LAST_KEY)
}

/// Makes a backup now and records the time.
pub fn backup_now(conn: &Connection, paths: &DbPaths, label: &str) -> Result<PathBuf, DbError> {
    let dir = backup_dir(conn, paths)?;
    let path = copy_backup(conn, &paths.db_file, &dir, label)?;
    settings::set(conn, LAST_KEY, &super::ids::now_iso())?;
    prune_daily(&dir)?;
    Ok(path)
}

/// Called by the agent. Backs up if the last one is a day old or more.
pub fn daily_if_due(conn: &Connection, paths: &DbPaths, now: DateTime<Utc>) -> Result<Option<PathBuf>, DbError> {
    let last = last_backup_at(conn)?
        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
        .map(|d| d.with_timezone(&Utc));
    if last.is_some_and(|l| now - l < Duration::hours(24)) {
        return Ok(None);
    }
    backup_now(conn, paths, "daily").map(Some)
}

#[derive(Debug, Serialize)]
pub struct BackupFile {
    pub name: String,
    pub bytes: u64,
}

/// MyLife backups in a folder, newest first.
pub fn list(dir: &Path) -> Result<Vec<BackupFile>, DbError> {
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut out: Vec<BackupFile> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let meta = e.metadata().ok()?;
            (name.starts_with(PREFIX) && name.ends_with(".db") && meta.is_file()).then_some(BackupFile { name, bytes: meta.len() })
        })
        .collect();
    // Names start with a UTC timestamp, so they sort by time.
    out.sort_by(|a, b| b.name.cmp(&a.name));
    Ok(out)
}

fn prune_daily(dir: &Path) -> Result<(), DbError> {
    let daily: Vec<_> = list(dir)?.into_iter().filter(|b| b.name.ends_with("-daily.db")).collect();
    for old in daily.iter().skip(KEEP_DAILY) {
        std::fs::remove_file(dir.join(&old.name))?;
    }
    Ok(())
}

/// Replaces the live database with a backup.
///
/// 1. Checks the backup opens with `password`.
/// 2. Backs up the current database ("prerestore").
/// 3. Closes, swaps the file, and reopens.
///
/// `slot` holds the open connection. It stays untouched if the check in
/// step 1 fails. After the swap it holds the reopened database.
pub fn restore(slot: &mut Option<Connection>, paths: &DbPaths, backup: &Path, password: &str) -> Result<(), DbError> {
    // Only files from the backup list may be restored.
    let name = backup.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    if !name.starts_with(PREFIX) || !name.ends_with(".db") {
        return Err(DbError::Invalid("That isn't a MyLife backup.".into()));
    }
    // 1. Verify on a scratch copy, so the backup itself is never changed.
    let scratch_dir = tempdir_in(&paths.backup_dir)?;
    let scratch = DbPaths { db_file: scratch_dir.join("check.db"), backup_dir: scratch_dir.join("b") };
    std::fs::copy(backup, &scratch.db_file)?;
    let check = super::unlock(&scratch, password);
    let _ = std::fs::remove_dir_all(&scratch_dir);
    check?;

    // 2. Safety backup of what we're about to replace, then close it.
    let conn = slot.as_ref().ok_or(DbError::Invalid("Unlock first.".into()))?;
    copy_backup(conn, &paths.db_file, &paths.backup_dir, "prerestore")?;
    drop(slot.take());

    // 3. Swap. Remove WAL files so SQLite doesn't replay old pages.
    for ext in ["-wal", "-shm"] {
        let side = PathBuf::from(format!("{}{ext}", paths.db_file.display()));
        if side.exists() {
            std::fs::remove_file(side)?;
        }
    }
    std::fs::copy(backup, &paths.db_file)?;
    *slot = Some(super::unlock(paths, password)?);
    Ok(())
}

fn tempdir_in(parent: &Path) -> Result<PathBuf, DbError> {
    let dir = parent.join(format!(".restore-check-{}", super::ids::new_id()));
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{create, unlock};

    fn setup() -> (tempfile::TempDir, DbPaths) {
        let dir = tempfile::tempdir().unwrap();
        let p = DbPaths::in_dir(dir.path());
        (dir, p)
    }

    fn count_items(c: &Connection) -> i64 {
        c.query_row("SELECT count(*) FROM items", [], |r| r.get(0)).unwrap()
    }

    fn add_item(c: &Connection, title: &str) {
        c.execute(
            "INSERT INTO items (id, module, type, title, created_at, updated_at) VALUES (?1,'tasks','task',?2,'t','t')",
            rusqlite::params![crate::db::ids::new_id(), title],
        )
        .unwrap();
    }

    #[test]
    fn restore_round_trip() {
        // Acceptance: a restore from backup is tested and works.
        let (_d, p) = setup();
        let c = create(&p, "correct horse").unwrap();
        add_item(&c, "before backup");
        let backup = backup_now(&c, &p, "daily").unwrap();
        add_item(&c, "after backup");
        assert_eq!(count_items(&c), 2);

        let mut slot = Some(c);
        restore(&mut slot, &p, &backup, "correct horse").unwrap();
        let c = slot.unwrap();
        assert_eq!(count_items(&c), 1);
        // The pre-restore safety copy holds the newer data.
        let pre = list(&p.backup_dir).unwrap().into_iter().find(|b| b.name.ends_with("-prerestore.db")).unwrap();
        drop(c);
        let pre_paths = DbPaths { db_file: p.backup_dir.join(pre.name), backup_dir: p.backup_dir.join("x") };
        assert_eq!(count_items(&unlock(&pre_paths, "correct horse").unwrap()), 2);
    }

    #[test]
    fn restore_needs_the_right_password() {
        let (_d, p) = setup();
        let c = create(&p, "correct horse").unwrap();
        let backup = backup_now(&c, &p, "daily").unwrap();
        add_item(&c, "keep me");
        let mut slot = Some(c);
        let err = restore(&mut slot, &p, &backup, "wrong horse").unwrap_err();
        assert!(matches!(err, DbError::WrongPassword));
        // Still open, live data untouched.
        assert_eq!(count_items(slot.as_ref().unwrap()), 1);
    }

    #[test]
    fn daily_runs_once_a_day_and_prunes() {
        let (_d, p) = setup();
        let c = create(&p, "correct horse").unwrap();
        let now = Utc::now();
        assert!(daily_if_due(&c, &p, now).unwrap().is_some());
        assert!(daily_if_due(&c, &p, now + Duration::hours(23)).unwrap().is_none());
        assert!(daily_if_due(&c, &p, now + Duration::hours(25)).unwrap().is_some());

        for i in 0..(KEEP_DAILY + 3) {
            std::fs::write(p.backup_dir.join(format!("mylife-2020010{i:02}T000000Z-daily.db")), b"x").unwrap();
        }
        backup_now(&c, &p, "daily").unwrap();
        let daily = list(&p.backup_dir).unwrap().into_iter().filter(|b| b.name.ends_with("-daily.db")).count();
        assert_eq!(daily, KEEP_DAILY);
    }

    #[test]
    fn chosen_folder_is_used() {
        let (d, p) = setup();
        let c = create(&p, "correct horse").unwrap();
        let other = d.path().join("OneDrive").join("MyLife backups");
        set_backup_dir(&c, &other).unwrap();
        backup_now(&c, &p, "daily").unwrap();
        assert_eq!(list(&other).unwrap().len(), 1);
    }

    #[test]
    fn rejects_files_that_are_not_backups() {
        let (d, p) = setup();
        let c = create(&p, "correct horse").unwrap();
        let other = d.path().join("notes.txt");
        std::fs::write(&other, b"hi").unwrap();
        let mut slot = Some(c);
        assert!(restore(&mut slot, &p, &other, "correct horse").is_err());
        assert!(slot.is_some());
    }
}
