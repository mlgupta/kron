pub mod clients;
pub mod reports;
pub mod sessions;
pub mod timer;

use tauri::State;

use crate::db::DbState;
use crate::error::{log_technical_error, AppError};

pub fn lock_db<'a>(
    state: &'a State<'_, DbState>,
) -> Result<std::sync::MutexGuard<'a, rusqlite::Connection>, String> {
    state.conn.lock().map_err(|error| {
        log_technical_error("database lock", &error);
        AppError::database().to_invoke_error()
    })
}
