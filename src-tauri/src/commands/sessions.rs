use chrono::Local;
use tauri::{AppHandle, State};

use crate::db::DbState;
use crate::models::TimeSession;
use crate::services::session_service;
use crate::timeutil::{self, parse_rfc3339};

use super::lock_db;

#[tauri::command]
pub fn get_sessions(
    state: State<'_, DbState>,
    client_id: Option<i64>,
    from_date: Option<String>,
    to_date: Option<String>,
) -> Result<Vec<TimeSession>, String> {
    let conn = lock_db(&state)?;
    let from = from_date
        .as_deref()
        .filter(|value| !value.is_empty())
        .map(timeutil::parse_date)
        .transpose()
        .map_err(|error| error.to_invoke_error())?;
    let to = to_date
        .as_deref()
        .filter(|value| !value.is_empty())
        .map(timeutil::parse_date)
        .transpose()
        .map_err(|error| error.to_invoke_error())?;
    session_service::get_sessions(&conn, client_id, from, to, &Local)
        .map_err(|error| error.to_invoke_error())
}

#[tauri::command]
pub fn get_session(state: State<'_, DbState>, id: i64) -> Result<TimeSession, String> {
    let conn = lock_db(&state)?;
    session_service::get_session(&conn, id).map_err(|error| error.to_invoke_error())
}

#[tauri::command]
pub fn update_session(
    app: AppHandle,
    state: State<'_, DbState>,
    id: i64,
    client_id: i64,
    started_at: String,
    stopped_at: String,
    note: Option<String>,
) -> Result<TimeSession, String> {
    let session = {
        let conn = lock_db(&state)?;
        let started = parse_rfc3339(&started_at).map_err(|error| error.to_invoke_error())?;
        let stopped = parse_rfc3339(&stopped_at).map_err(|error| error.to_invoke_error())?;
        session_service::update_session(&conn, id, client_id, started, stopped, note.as_deref())
            .map_err(|error| error.to_invoke_error())?
    };
    if let Err(error) = crate::tray::refresh(&app) {
        crate::error::log_technical_error("refresh tray", &error);
    }
    Ok(session)
}

#[tauri::command]
pub fn delete_session(app: AppHandle, state: State<'_, DbState>, id: i64) -> Result<(), String> {
    {
        let conn = lock_db(&state)?;
        session_service::delete_session(&conn, id).map_err(|error| error.to_invoke_error())?;
    }
    if let Err(error) = crate::tray::refresh(&app) {
        crate::error::log_technical_error("refresh tray", &error);
    }
    Ok(())
}
