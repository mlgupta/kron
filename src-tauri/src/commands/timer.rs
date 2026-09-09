use tauri::{AppHandle, State};

use crate::db::DbState;
use crate::models::{ActiveTimer, TimeSession};
use crate::services::timer_service;

use super::lock_db;

#[tauri::command]
pub fn start_timer(
    app: AppHandle,
    state: State<'_, DbState>,
    client_id: i64,
    switch_client: bool,
) -> Result<ActiveTimer, String> {
    let timer = {
        let conn = lock_db(&state)?;
        timer_service::start_timer(&conn, client_id, None, switch_client)
            .map_err(|error| error.to_invoke_error())?
    };
    crate::tray::notify_timer_changed(&app);
    Ok(timer)
}

#[tauri::command]
pub fn stop_timer(app: AppHandle, state: State<'_, DbState>) -> Result<TimeSession, String> {
    let session = {
        let conn = lock_db(&state)?;
        timer_service::stop_timer(&conn).map_err(|error| error.to_invoke_error())?
    };
    crate::tray::notify_timer_changed(&app);
    Ok(session)
}

#[tauri::command]
pub fn get_active_timer(state: State<'_, DbState>) -> Result<Option<ActiveTimer>, String> {
    let conn = lock_db(&state)?;
    timer_service::get_active_timer(&conn).map_err(|error| error.to_invoke_error())
}

#[tauri::command]
pub fn is_timer_running(state: State<'_, DbState>) -> Result<bool, String> {
    let conn = lock_db(&state)?;
    timer_service::is_timer_running(&conn).map_err(|error| error.to_invoke_error())
}
