pub mod migrations;

use std::path::PathBuf;
use std::sync::Mutex;

use rusqlite::Connection;
use tauri::{AppHandle, Manager};

use crate::error::{log_technical_error, AppError};

pub struct DbState {
    pub conn: Mutex<Connection>,
}

pub fn initialize(app: &AppHandle) -> Result<DbState, AppError> {
    let db_path = database_path(app)?;
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            log_technical_error("create app data directory", &error);
            AppError::database()
        })?;
    }

    let conn = open_connection(&db_path)?;
    Ok(DbState {
        conn: Mutex::new(conn),
    })
}

pub fn open_connection(path: &PathBuf) -> Result<Connection, AppError> {
    let conn = Connection::open(path).map_err(|error| {
        log_technical_error("open sqlite database", &error);
        AppError::database()
    })?;
    configure(&conn, true)?;
    migrations::run(&conn)?;
    Ok(conn)
}

pub fn open_in_memory() -> Result<Connection, AppError> {
    let conn = Connection::open_in_memory().map_err(|error| {
        log_technical_error("open in-memory sqlite database", &error);
        AppError::database()
    })?;
    configure(&conn, false)?;
    migrations::run(&conn)?;
    Ok(conn)
}

fn configure(conn: &Connection, persistent: bool) -> Result<(), AppError> {
    conn.pragma_update(None, "foreign_keys", "ON")?;
    if persistent {
        conn.pragma_update(None, "journal_mode", "WAL")?;
    }
    Ok(())
}

pub fn database_path(app: &AppHandle) -> Result<PathBuf, AppError> {
    let mut dir = app.path().app_data_dir().map_err(|error| {
        log_technical_error("resolve app data directory", &error);
        AppError::database()
    })?;
    dir.push("kron.db");
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_database_enables_foreign_keys() {
        let conn = open_in_memory().unwrap();
        let enabled: i64 = conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();
        assert_eq!(enabled, 1);
    }
}
