use tauri::{AppHandle, State};

use crate::db::DbState;
use crate::models::Client;
use crate::services::client_service;

use super::lock_db;

fn refresh_tray(app: &AppHandle) {
    if let Err(error) = crate::tray::refresh(app) {
        crate::error::log_technical_error("refresh tray", &error);
    }
}

#[tauri::command]
pub fn create_client(
    app: AppHandle,
    state: State<'_, DbState>,
    name: String,
    description: Option<String>,
) -> Result<Client, String> {
    let client = {
        let conn = lock_db(&state)?;
        client_service::create_client(&conn, &name, description.as_deref())
            .map_err(|error| error.to_invoke_error())?
    };
    refresh_tray(&app);
    Ok(client)
}

#[tauri::command]
pub fn get_clients(state: State<'_, DbState>) -> Result<Vec<Client>, String> {
    let conn = lock_db(&state)?;
    client_service::get_clients(&conn).map_err(|error| error.to_invoke_error())
}

#[tauri::command]
pub fn get_client(state: State<'_, DbState>, id: i64) -> Result<Client, String> {
    let conn = lock_db(&state)?;
    client_service::get_client(&conn, id).map_err(|error| error.to_invoke_error())
}

#[tauri::command]
pub fn update_client(
    app: AppHandle,
    state: State<'_, DbState>,
    id: i64,
    name: String,
    description: Option<String>,
) -> Result<Client, String> {
    let client = {
        let conn = lock_db(&state)?;
        client_service::update_client(&conn, id, &name, description.as_deref())
            .map_err(|error| error.to_invoke_error())?
    };
    refresh_tray(&app);
    Ok(client)
}

#[tauri::command]
pub fn set_client_active(
    app: AppHandle,
    state: State<'_, DbState>,
    id: i64,
    active: bool,
) -> Result<Client, String> {
    let client = {
        let conn = lock_db(&state)?;
        client_service::set_client_active(&conn, id, active).map_err(|error| error.to_invoke_error())?
    };
    refresh_tray(&app);
    Ok(client)
}

#[tauri::command]
pub fn delete_client(
    app: AppHandle,
    state: State<'_, DbState>,
    id: i64,
    force: bool,
) -> Result<(), String> {
    {
        let conn = lock_db(&state)?;
        client_service::delete_client(&conn, id, force).map_err(|error| error.to_invoke_error())?;
    }
    refresh_tray(&app);
    Ok(())
}
