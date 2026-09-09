use chrono::Local;
use tauri::State;

use crate::db::DbState;
use crate::models::Report;
use crate::services::report_service;
use crate::timeutil;

use super::lock_db;

#[tauri::command]
pub fn get_daily_totals(state: State<'_, DbState>, date: String) -> Result<Report, String> {
    let conn = lock_db(&state)?;
    let parsed = timeutil::parse_date(&date).map_err(|error| error.to_invoke_error())?;
    report_service::get_daily_totals(&conn, parsed, &Local).map_err(|error| error.to_invoke_error())
}

#[tauri::command]
pub fn get_weekly_totals(state: State<'_, DbState>, date: String) -> Result<Report, String> {
    let conn = lock_db(&state)?;
    let parsed = timeutil::parse_date(&date).map_err(|error| error.to_invoke_error())?;
    report_service::get_weekly_totals(&conn, parsed, &Local).map_err(|error| error.to_invoke_error())
}

#[tauri::command]
pub fn get_monthly_totals(state: State<'_, DbState>, date: String) -> Result<Report, String> {
    let conn = lock_db(&state)?;
    let parsed = timeutil::parse_date(&date).map_err(|error| error.to_invoke_error())?;
    report_service::get_monthly_totals(&conn, parsed, &Local)
        .map_err(|error| error.to_invoke_error())
}
