use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};

use crate::error::AppError;
use crate::models::{ActiveTimer, TimeSession};
use crate::rounding::round_up_from_seconds;
use crate::services::client_service;
use crate::services::session_service;
use crate::timeutil::parse_rfc3339;

pub fn start_timer(
    conn: &Connection,
    client_id: i64,
    note: Option<&str>,
    switch: bool,
) -> Result<ActiveTimer, AppError> {
    start_timer_at(conn, client_id, note, Utc::now(), switch)
}

pub fn start_timer_at(
    conn: &Connection,
    client_id: i64,
    note: Option<&str>,
    started_at: DateTime<Utc>,
    switch: bool,
) -> Result<ActiveTimer, AppError> {
    let client = client_service::get_client(conn, client_id)?;
    if !client.active {
        return Err(AppError::client_inactive());
    }

    let tx = conn.unchecked_transaction()?;
    let current = fetch_active(&tx)?;

    if let Some(current) = current {
        if current.client_id == client_id {
            return Ok(current);
        }
        if !switch {
            return Err(AppError::timer_already_running());
        }
        persist_active_session(&tx, &current, started_at)?;
        clear_active(&tx)?;
    }

    let note = normalize_note(note);
    tx.execute(
        "INSERT INTO active_timer (id, client_id, started_at, note)
         VALUES (1, ?1, ?2, ?3)",
        params![client_id, started_at.to_rfc3339(), note],
    )?;
    tx.commit()?;

    get_active_timer(conn)?.ok_or_else(AppError::database)
}

pub fn stop_timer(conn: &Connection) -> Result<TimeSession, AppError> {
    stop_timer_at(conn, Utc::now())
}

pub fn stop_timer_at(
    conn: &Connection,
    stopped_at: DateTime<Utc>,
) -> Result<TimeSession, AppError> {
    let tx = conn.unchecked_transaction()?;
    let current = fetch_active(&tx)?.ok_or_else(AppError::timer_not_running)?;
    let session_id = persist_active_session(&tx, &current, stopped_at)?;
    clear_active(&tx)?;
    tx.commit()?;
    session_service::get_session(conn, session_id)
}

pub fn get_active_timer(conn: &Connection) -> Result<Option<ActiveTimer>, AppError> {
    fetch_active(conn)
}

pub fn is_timer_running(conn: &Connection) -> Result<bool, AppError> {
    Ok(get_active_timer(conn)?.is_some())
}

fn fetch_active(conn: &Connection) -> Result<Option<ActiveTimer>, AppError> {
    Ok(conn
        .query_row(
            "SELECT at.client_id, c.name, at.started_at, at.note
             FROM active_timer at
             JOIN clients c ON c.id = at.client_id
             WHERE at.id = 1",
            [],
            |row| {
                Ok(ActiveTimer {
                    client_id: row.get(0)?,
                    client_name: row.get(1)?,
                    started_at: row.get(2)?,
                    note: row.get(3)?,
                })
            },
        )
        .optional()?)
}

fn persist_active_session(
    conn: &Connection,
    active: &ActiveTimer,
    stopped_at: DateTime<Utc>,
) -> Result<i64, AppError> {
    let started_at = parse_rfc3339(&active.started_at)?;
    let actual_duration_seconds = (stopped_at - started_at).num_seconds().max(0);
    let rounded_duration_minutes = round_up_from_seconds(actual_duration_seconds);
    let created_at = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO time_sessions (
            client_id, started_at, stopped_at,
            actual_duration_seconds, rounded_duration_minutes,
            note, created_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            active.client_id,
            active.started_at,
            stopped_at.to_rfc3339(),
            actual_duration_seconds,
            rounded_duration_minutes,
            active.note,
            created_at,
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

fn clear_active(conn: &Connection) -> Result<(), AppError> {
    conn.execute("DELETE FROM active_timer WHERE id = 1", [])?;
    Ok(())
}

fn normalize_note(note: Option<&str>) -> Option<String> {
    note.map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::services::client_service;
    use chrono::TimeZone;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn setup() -> Connection {
        db::open_in_memory().expect("in-memory database")
    }

    fn at(hour: u32, minute: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 9, 5, hour, minute, 0).unwrap()
    }

    #[test]
    fn starts_and_reads_active_timer() {
        let conn = setup();
        let client = client_service::create_client(&conn, "Acme", None).unwrap();

        let active = start_timer_at(&conn, client.id, None, at(9, 2), false).unwrap();

        assert_eq!(active.client_id, client.id);
        assert_eq!(active.client_name, "Acme");
        assert!(is_timer_running(&conn).unwrap());
        assert_eq!(get_active_timer(&conn).unwrap().unwrap().client_id, client.id);
    }

    #[test]
    fn refuses_second_timer_without_switch() {
        let conn = setup();
        let acme = client_service::create_client(&conn, "Acme", None).unwrap();
        let globex = client_service::create_client(&conn, "Globex", None).unwrap();
        start_timer_at(&conn, acme.id, None, at(9, 0), false).unwrap();

        let error = start_timer_at(&conn, globex.id, None, at(10, 0), false).unwrap_err();
        assert_eq!(error.code, "TIMER_ALREADY_RUNNING");
        assert_eq!(get_active_timer(&conn).unwrap().unwrap().client_id, acme.id);
    }

    #[test]
    fn switching_clients_stops_current_and_starts_next() {
        let conn = setup();
        let acme = client_service::create_client(&conn, "Acme", None).unwrap();
        let globex = client_service::create_client(&conn, "Globex", None).unwrap();
        start_timer_at(&conn, acme.id, None, at(9, 0), false).unwrap();

        let active = start_timer_at(&conn, globex.id, None, at(9, 16), true).unwrap();
        assert_eq!(active.client_id, globex.id);

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM time_sessions", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);

        let minutes: i64 = conn
            .query_row(
                "SELECT rounded_duration_minutes FROM time_sessions WHERE client_id = ?1",
                [acme.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(minutes, 30);
    }

    #[test]
    fn starting_same_client_is_idempotent() {
        let conn = setup();
        let client = client_service::create_client(&conn, "Acme", None).unwrap();
        let first = start_timer_at(&conn, client.id, None, at(9, 0), false).unwrap();
        let second = start_timer_at(&conn, client.id, None, at(10, 0), false).unwrap();
        assert_eq!(first.started_at, second.started_at);
    }

    #[test]
    fn stop_persists_rounded_session() {
        let conn = setup();
        let client = client_service::create_client(&conn, "Acme", None).unwrap();
        start_timer_at(&conn, client.id, Some(" research "), at(9, 0), false).unwrap();

        let session = stop_timer_at(&conn, at(9, 16)).unwrap();
        assert_eq!(session.actual_duration_seconds, 16 * 60);
        assert_eq!(session.rounded_duration_minutes, 30);
        assert_eq!(session.note.as_deref(), Some("research"));
        assert!(!is_timer_running(&conn).unwrap());
    }

    #[test]
    fn stop_without_timer_fails() {
        let conn = setup();
        let error = stop_timer(&conn).unwrap_err();
        assert_eq!(error.code, "TIMER_NOT_RUNNING");
    }

    #[test]
    fn inactive_client_cannot_start_timer() {
        let conn = setup();
        let client = client_service::create_client(&conn, "Acme", None).unwrap();
        client_service::set_client_active(&conn, client.id, false).unwrap();
        let error = start_timer(&conn, client.id, None, false).unwrap_err();
        assert_eq!(error.code, "CLIENT_INACTIVE");
    }

    #[test]
    fn active_timer_survives_reopen() {
        let path = std::env::temp_dir().join(format!(
            "kron-timer-persist-{}-{}.db",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let client_id;
        let started_at = at(9, 2).to_rfc3339();

        {
            let conn = db::open_connection(&path).unwrap();
            let client = client_service::create_client(&conn, "Acme", None).unwrap();
            client_id = client.id;
            start_timer_at(&conn, client.id, None, at(9, 2), false).unwrap();
        }

        let conn = db::open_connection(&path).unwrap();
        let active = get_active_timer(&conn).unwrap().expect("timer restored");
        assert_eq!(active.client_id, client_id);
        assert_eq!(active.started_at, started_at);

        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(format!("{}-wal", path.display()));
        let _ = std::fs::remove_file(format!("{}-shm", path.display()));
    }

    #[test]
    fn only_one_active_timer_row() {
        let conn = setup();
        let acme = client_service::create_client(&conn, "Acme", None).unwrap();
        start_timer_at(&conn, acme.id, None, at(9, 0), false).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM active_timer", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }
}
