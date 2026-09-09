pub mod commands;
pub mod db;
pub mod error;
pub mod models;
pub mod rounding;
pub mod services;
pub mod timeutil;
pub mod tray;

use error::log_technical_error;
use tauri::Manager;

#[tauri::command]
fn ping() -> &'static str {
    "ok"
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let db = db::initialize(app.handle()).inspect_err(|error| {
                log_technical_error("initialize database", error);
            })?;
            app.manage(db);
            tray::setup(app.handle()).inspect_err(|error| {
                log_technical_error("initialize tray", error);
            })?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            ping,
            commands::clients::create_client,
            commands::clients::get_clients,
            commands::clients::get_client,
            commands::clients::update_client,
            commands::clients::set_client_active,
            commands::clients::delete_client,
            commands::timer::start_timer,
            commands::timer::stop_timer,
            commands::timer::get_active_timer,
            commands::timer::is_timer_running,
            commands::sessions::get_sessions,
            commands::sessions::get_session,
            commands::sessions::update_session,
            commands::sessions::delete_session,
            commands::reports::get_daily_totals,
            commands::reports::get_weekly_totals,
            commands::reports::get_monthly_totals,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
