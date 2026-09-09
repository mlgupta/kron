use std::sync::Mutex;
use std::time::Duration;

use chrono::Local;
use rusqlite::Connection;
use tauri::image::Image;
use tauri::menu::{IsMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::TrayIconBuilder;
#[cfg(windows)]
use tauri::tray::{MouseButton, MouseButtonState, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, Wry};

use crate::db::DbState;
use crate::error::log_technical_error;
use crate::models::{ActiveTimer, Client};
use crate::services::{client_service, report_service, timer_service};
use crate::timeutil::{self, parse_rfc3339};

const TRAY_ID: &str = "main";
const EVENT_TIMER_CHANGED: &str = "timer-changed";

/// Monochrome glyph so macOS can tint it for light and dark menu bars.
#[cfg(target_os = "macos")]
const TRAY_ICON_PNG: &[u8] = include_bytes!("../icons/tray.png");

/// Color icon for Windows/Linux, where template (mask) icons are not supported.
#[cfg(not(target_os = "macos"))]
const TRAY_ICON_PNG: &[u8] = include_bytes!("../icons/32x32.png");

struct TraySnapshot {
    active: Option<ActiveTimer>,
    clients: Vec<Client>,
    today_total_minutes: i64,
}

/// Handle to the elapsed row, so the ticker can retitle it in place. Replacing the whole
/// menu on a timer would dismiss it out from under the user while it is open.
#[derive(Default)]
struct ElapsedRow(Mutex<Option<MenuItem<Wry>>>);

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    app.manage(ElapsedRow::default());

    let menu = build_menu(app)?;
    let title = current_title(app);
    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .tooltip(&title)
        .on_menu_event(|app, event| handle_menu_event(app, event.id.as_ref()));

    #[cfg(target_os = "macos")]
    {
        builder = builder.title(&title).show_menu_on_left_click(true);
    }

    #[cfg(windows)]
    {
        builder = builder.show_menu_on_left_click(false).on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        });
    }

    match Image::from_bytes(TRAY_ICON_PNG) {
        Ok(icon) => {
            builder = builder.icon(icon);
            #[cfg(target_os = "macos")]
            {
                builder = builder.icon_as_template(true);
            }
        }
        Err(error) => log_technical_error("load tray icon", &error),
    }

    builder.build(app)?;

    let handle = app.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(1));
        tick(&handle);
    });

    Ok(())
}

/// Refreshes everything, including the menu structure. Only safe when the menu is closed,
/// so call this on state changes rather than on tray clicks.
pub fn refresh(app: &AppHandle) -> tauri::Result<()> {
    apply_status(app)?;
    apply_menu(app)
}

pub fn notify_timer_changed(app: &AppHandle) {
    if let Err(error) = refresh(app) {
        log_technical_error("refresh tray", &error);
    }
    if let Err(error) = app.emit(EVENT_TIMER_CHANGED, ()) {
        log_technical_error("emit timer-changed", &error);
    }
}

/// Retitles the tray and the elapsed row without touching menu structure, so the menu
/// keeps counting up while the user has it open.
fn tick(app: &AppHandle) {
    let active = with_db(app, |conn| timer_service::get_active_timer(conn).ok().flatten()).flatten();
    let elapsed = active.as_ref().map(|timer| elapsed_for(timer).unwrap_or(0));

    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let title = timeutil::tray_title(
            active.as_ref().map(|timer| timer.client_name.as_str()),
            elapsed.unwrap_or(0),
        );
        apply_tray_status(&tray, &title);
    }

    if let (Some(seconds), Some(row)) = (elapsed, elapsed_row(app)) {
        if let Err(error) = row.set_text(timeutil::format_elapsed_hms(seconds)) {
            log_technical_error("update elapsed row", &error);
        }
    }
}

fn set_elapsed_row(app: &AppHandle, item: Option<MenuItem<Wry>>) {
    if let Some(state) = app.try_state::<ElapsedRow>() {
        match state.0.lock() {
            Ok(mut slot) => *slot = item,
            Err(error) => log_technical_error("elapsed row lock", &error),
        }
    }
}

fn elapsed_row(app: &AppHandle) -> Option<MenuItem<Wry>> {
    let state = app.try_state::<ElapsedRow>()?;
    let slot = state.0.lock().ok()?;
    slot.clone()
}

fn apply_status(app: &AppHandle) -> tauri::Result<()> {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        apply_tray_status(&tray, &current_title(app));
    }
    Ok(())
}

fn apply_tray_status(tray: &tauri::tray::TrayIcon<Wry>, title: &str) {
    if let Err(error) = tray.set_tooltip(Some(title)) {
        log_technical_error("update tray tooltip", &error);
    }
    #[cfg(target_os = "macos")]
    if let Err(error) = tray.set_title(Some(title)) {
        log_technical_error("update tray title", &error);
    }
}

fn apply_menu(app: &AppHandle) -> tauri::Result<()> {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let menu = build_menu(app)?;
        tray.set_menu(Some(menu))?;
    }
    Ok(())
}

fn current_title(app: &AppHandle) -> String {
    match with_db(app, |conn| timer_service::get_active_timer(conn)) {
        Some(Ok(Some(active))) => {
            let elapsed = elapsed_for(&active).unwrap_or(0);
            timeutil::tray_title(Some(&active.client_name), elapsed)
        }
        _ => timeutil::tray_title(None, 0),
    }
}

fn snapshot(app: &AppHandle) -> TraySnapshot {
    with_db(app, |conn| {
        let active = timer_service::get_active_timer(conn).ok().flatten();
        let clients = client_service::get_clients(conn).unwrap_or_default();
        let today_total_minutes = report_service::get_daily_totals(conn, Local::now().date_naive(), &Local)
            .map(|report| report.total_rounded_minutes)
            .unwrap_or(0);
        TraySnapshot {
            active,
            clients,
            today_total_minutes,
        }
    })
    .unwrap_or(TraySnapshot {
        active: None,
        clients: Vec::new(),
        today_total_minutes: 0,
    })
}

fn build_menu(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
    let TraySnapshot {
        active,
        clients,
        today_total_minutes,
    } = snapshot(app);
    let active_clients: Vec<&Client> = clients.iter().filter(|client| client.active).collect();
    let separator = PredefinedMenuItem::separator(app)?;
    let separator_after_actions = PredefinedMenuItem::separator(app)?;
    let separator_after_today = PredefinedMenuItem::separator(app)?;
    let today = MenuItem::with_id(
        app,
        "today-total",
        format!(
            "Today's total: {}",
            timeutil::format_hours_minutes(today_total_minutes)
        ),
        false,
        None::<&str>,
    )?;
    let open = MenuItem::with_id(app, "open", "Open Kron", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    if let Some(active) = active {
        let elapsed = elapsed_for(&active).unwrap_or(0);
        let client_label =
            MenuItem::with_id(app, "active-client", &active.client_name, false, None::<&str>)?;
        let elapsed_label = MenuItem::with_id(
            app,
            "elapsed",
            timeutil::format_elapsed_hms(elapsed),
            false,
            None::<&str>,
        )?;
        set_elapsed_row(app, Some(elapsed_label.clone()));
        let stop = MenuItem::with_id(app, "stop-timer", "Stop", true, None::<&str>)?;
        let others: Vec<&Client> = active_clients
            .iter()
            .copied()
            .filter(|client| client.id != active.client_id)
            .collect();
        let switch_items = client_menu_items(app, &others, "switch")?;
        let switch_empty = MenuItem::with_id(
            app,
            "switch-empty",
            "No other clients",
            false,
            None::<&str>,
        )?;
        let switch_refs = menu_item_refs(&switch_items, &switch_empty);
        let switch = Submenu::with_items(app, "Switch Client", true, &switch_refs)?;
        Menu::with_items(
            app,
            &[
                &client_label,
                &elapsed_label,
                &stop,
                &separator,
                &switch,
                &separator_after_actions,
                &today,
                &separator_after_today,
                &open,
                &quit,
            ],
        )
    } else {
        set_elapsed_row(app, None);
        let start_items = client_menu_items(app, &active_clients, "start")?;
        let start_empty = MenuItem::with_id(
            app,
            "start-empty",
            "No active clients",
            false,
            None::<&str>,
        )?;
        let start_refs = menu_item_refs(&start_items, &start_empty);
        let start = Submenu::with_items(app, "Start Timer", true, &start_refs)?;
        Menu::with_items(
            app,
            &[
                &start,
                &separator,
                &today,
                &separator_after_today,
                &open,
                &quit,
            ],
        )
    }
}

fn client_menu_items(
    app: &AppHandle,
    clients: &[&Client],
    prefix: &str,
) -> tauri::Result<Vec<MenuItem<Wry>>> {
    clients
        .iter()
        .map(|client| {
            MenuItem::with_id(
                app,
                format!("{}-{}", prefix, client.id),
                &client.name,
                true,
                None::<&str>,
            )
        })
        .collect()
}

fn menu_item_refs<'a>(
    items: &'a [MenuItem<Wry>],
    empty: &'a MenuItem<Wry>,
) -> Vec<&'a dyn IsMenuItem<Wry>> {
    if items.is_empty() {
        vec![empty]
    } else {
        items.iter().map(|item| item as &dyn IsMenuItem<Wry>).collect()
    }
}

fn handle_menu_event(app: &AppHandle, id: &str) {
    if id == "stop-timer" {
        if let Some(Err(error)) = with_db(app, |conn| timer_service::stop_timer(conn)) {
            log_technical_error("stop timer from tray", &error);
        }
        notify_timer_changed(app);
        return;
    }
    if id == "open" {
        show_main_window(app);
        return;
    }
    if id == "quit" {
        app.exit(0);
        return;
    }
    if let Some(client_id) = parse_prefixed_id(id, "start-") {
        start_from_tray(app, client_id, false);
        return;
    }
    if let Some(client_id) = parse_prefixed_id(id, "switch-") {
        start_from_tray(app, client_id, true);
    }
}

fn start_from_tray(app: &AppHandle, client_id: i64, switch_client: bool) {
    if let Some(Err(error)) = with_db(app, |conn| {
        timer_service::start_timer(conn, client_id, None, switch_client)
    }) {
        log_technical_error("start timer from tray", &error);
    }
    notify_timer_changed(app);
}

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn parse_prefixed_id(value: &str, prefix: &str) -> Option<i64> {
    value.strip_prefix(prefix)?.parse().ok()
}

fn elapsed_for(active: &ActiveTimer) -> Result<i64, crate::error::AppError> {
    let started = parse_rfc3339(&active.started_at)?;
    Ok(timeutil::elapsed_seconds_since(started))
}

fn with_db<T>(app: &AppHandle, callback: impl FnOnce(&Connection) -> T) -> Option<T> {
    let db = app.try_state::<DbState>()?;
    let conn = match db.conn.lock() {
        Ok(conn) => conn,
        Err(error) => {
            log_technical_error("database lock", &error);
            return None;
        }
    };
    Some(callback(&conn))
}
