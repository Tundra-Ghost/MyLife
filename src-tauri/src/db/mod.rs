//! The main database, `mylife.db`, encrypted with SQLCipher.
//!
//! The app password is the SQLCipher key. Without it the file cannot be
//! read, even with a normal SQLite tool.

pub mod backup;
pub mod ids;
pub mod migrations;
pub mod settings;

use rusqlite::Connection;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("Wrong password.")]
    WrongPassword,
    #[error("Password must be at least {0} characters.")]
    PasswordTooShort(usize),
    #[error("A database already exists here.")]
    AlreadyExists,
    #[error("No database yet. Create a password first.")]
    NotCreated,
    #[error("Not found.")]
    NotFound,
    #[error("{0}")]
    Invalid(String),
    #[error("Database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("File error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Data error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Minimum app password length. A default, recorded in docs/DECISIONS.md.
pub const MIN_PASSWORD_LEN: usize = 8;

/// Where the app keeps its files.
#[derive(Clone, Debug)]
pub struct DbPaths {
    pub db_file: PathBuf,
    pub backup_dir: PathBuf,
}

impl DbPaths {
    pub fn in_dir(dir: &Path) -> Self {
        DbPaths {
            db_file: dir.join("mylife.db"),
            backup_dir: dir.join("backups"),
        }
    }

    pub fn exists(&self) -> bool {
        self.db_file.exists()
    }
}

/// First run: creates a new encrypted database with this password.
pub fn create(paths: &DbPaths, password: &str) -> Result<Connection, DbError> {
    if password.chars().count() < MIN_PASSWORD_LEN {
        return Err(DbError::PasswordTooShort(MIN_PASSWORD_LEN));
    }
    if paths.exists() {
        return Err(DbError::AlreadyExists);
    }
    if let Some(parent) = paths.db_file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    open_keyed(paths, password)
}

/// Unlocks an existing database. Returns `WrongPassword` if the key is wrong.
pub fn unlock(paths: &DbPaths, password: &str) -> Result<Connection, DbError> {
    if !paths.exists() {
        return Err(DbError::NotCreated);
    }
    open_keyed(paths, password)
}

fn open_keyed(paths: &DbPaths, password: &str) -> Result<Connection, DbError> {
    let mut conn = Connection::open(&paths.db_file)?;
    // SQLCipher derives the AES-256 key from the password (PBKDF2-HMAC-SHA512).
    // pragma_update quotes the value, so any characters are safe.
    conn.pragma_update(None, "key", password)?;
    // The first real read fails if the key is wrong.
    if conn
        .query_row("SELECT count(*) FROM sqlite_master", [], |r| r.get::<_, i64>(0))
        .is_err()
    {
        return Err(DbError::WrongPassword);
    }
    conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")?;
    migrations::migrate(&mut conn, &paths.db_file, &paths.backup_dir)?;
    Ok(conn)
}

/// An unencrypted in-memory database with the full schema, for tests.
#[cfg(test)]
pub fn test_conn() -> Connection {
    let mut conn = Connection::open_in_memory().unwrap();
    for (_, sql) in migrations::MIGRATIONS {
        let tx = conn.transaction().unwrap();
        tx.execute_batch(sql).unwrap();
        tx.commit().unwrap();
    }
    conn
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths() -> (tempfile::TempDir, DbPaths) {
        let dir = tempfile::tempdir().unwrap();
        let p = DbPaths::in_dir(dir.path());
        (dir, p)
    }

    #[test]
    fn create_then_unlock() {
        let (_d, p) = paths();
        let conn = create(&p, "correct horse").unwrap();
        assert_eq!(migrations::current_version(&conn).unwrap(), migrations::latest_version());
        drop(conn);
        assert!(unlock(&p, "correct horse").is_ok());
    }

    #[test]
    fn wrong_password_fails() {
        let (_d, p) = paths();
        drop(create(&p, "correct horse").unwrap());
        assert!(matches!(unlock(&p, "wrong horse"), Err(DbError::WrongPassword)));
    }

    #[test]
    fn short_password_rejected() {
        let (_d, p) = paths();
        assert!(matches!(create(&p, "short"), Err(DbError::PasswordTooShort(_))));
        assert!(!p.exists());
    }

    #[test]
    fn file_is_unreadable_without_key() {
        // Acceptance: mylife.db cannot be opened without the password.
        let (_d, p) = paths();
        let conn = create(&p, "correct horse").unwrap();
        conn.execute(
            "INSERT INTO items (id, module, type, title, created_at, updated_at) VALUES ('x','tasks','task','secret plans','t','t')",
            [],
        )
        .unwrap();
        conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);").unwrap();
        drop(conn);
        // Opening with no key, like a plain SQLite tool, must fail.
        let plain = Connection::open(&p.db_file).unwrap();
        assert!(plain.query_row("SELECT count(*) FROM sqlite_master", [], |r| r.get::<_, i64>(0)).is_err());
        // And the text must not appear in the raw bytes.
        let bytes = std::fs::read(&p.db_file).unwrap();
        assert!(!bytes.windows(12).any(|w| w == b"secret plans"));
    }

    #[test]
    fn backup_copy_is_still_encrypted() {
        let (_d, p) = paths();
        let conn = create(&p, "correct horse").unwrap();
        let dest = backup::copy_backup(&conn, &p.db_file, &p.backup_dir, "test").unwrap();
        drop(conn);
        let bp = DbPaths { db_file: dest, backup_dir: p.backup_dir.clone() };
        assert!(matches!(unlock(&bp, "nope nope"), Err(DbError::WrongPassword)));
        assert!(unlock(&bp, "correct horse").is_ok());
    }

    #[test]
    fn fts5_is_available() {
        let conn = test_conn();
        conn.execute("INSERT INTO search_index (item_id, module, title, body) VALUES ('1','tasks','Oil change','')", [])
            .unwrap();
        let n: i64 = conn
            .query_row("SELECT count(*) FROM search_index WHERE search_index MATCH 'oil'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 1);
    }
}
