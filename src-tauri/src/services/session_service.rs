use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use rusqlite::{params, Connection, OptionalExtension};

use crate::error::AppError;
use crate::models::TimeSession;
use crate::rounding::round_up_from_seconds;
use crate::services::client_service;
use crate::timeutil::{self, parse_rfc3339};

const SESSION_SELECT: &str = "SELECT s.id, s.client_id, c.name, s.started_at, s.stopped_at,
                s.actual_duration_seconds, s.rounded_duration_minutes,
                s.note, s.created_at
         FROM time_sessions s
         JOIN clients c ON c.id = s.client_id";

pub fn get_session(conn: &Connection, id: i64) -> Result<TimeSession, AppError> {
    conn.query_row(
        &format!("{SESSION_SELECT} WHERE s.id = ?1"),
        [id],
        map_session,
    )
    .optional()?
    .ok_or_else(|| AppError::not_found("That time session could not be found."))
}

pub fn get_sessions<Tz: TimeZone>(
    conn: &Connection,
    client_id: Option<i64>,
    from_date: Option<NaiveDate>,
    to_date: Option<NaiveDate>,
    tz: &Tz,
) -> Result<Vec<TimeSession>, AppError> {
    if let (Some(from), Some(to)) = (from_date, to_date) {
        if from > to {
            return Err(AppError::invalid_date_range());
        }
    }

    let mut sql = SESSION_SELECT.to_string();
    sql.push_str(" WHERE 1 = 1");
    if client_id.is_some() {
        sql.push_str(" AND s.client_id = ?1");
    }
    sql.push_str(" ORDER BY s.started_at ASC, s.id ASC");

    let mut stmt = conn.prepare(&sql)?;
    let rows = if let Some(client_id) = client_id {
        stmt.query_map([client_id], map_session)?
            .collect::<Result<Vec<_>, _>>()?
    } else {
        stmt.query_map([], map_session)?
            .collect::<Result<Vec<_>, _>>()?
    };

    Ok(rows
        .into_iter()
        .filter(|session| {
            let started = parse_rfc3339(&session.started_at).ok();
            let Some(started) = started else {
                return false;
            };
            let local = timeutil::local_date(started, tz);
            from_date.is_none_or(|from| local >= from) && to_date.is_none_or(|to| local <= to)
        })
        .collect())
}

pub fn insert_session(
    conn: &Connection,
    client_id: i64,
    started_at: DateTime<Utc>,
    stopped_at: DateTime<Utc>,
    note: Option<&str>,
) -> Result<TimeSession, AppError> {
    client_service::get_client(conn, client_id)?;
    if stopped_at <= started_at {
        return Err(AppError::invalid_session());
    }

    let actual_duration_seconds = (stopped_at - started_at).num_seconds();
    let rounded_duration_minutes = round_up_from_seconds(actual_duration_seconds);
    let created_at = Utc::now().to_rfc3339();
    let note = normalize_note(note);

    conn.execute(
        "INSERT INTO time_sessions (
            client_id, started_at, stopped_at,
            actual_duration_seconds, rounded_duration_minutes,
            note, created_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            client_id,
            started_at.to_rfc3339(),
            stopped_at.to_rfc3339(),
            actual_duration_seconds,
            rounded_duration_minutes,
            note,
            created_at,
        ],
    )?;

    get_session(conn, conn.last_insert_rowid())
}

pub fn update_session(
    conn: &Connection,
    id: i64,
    client_id: i64,
    started_at: DateTime<Utc>,
    stopped_at: DateTime<Utc>,
    note: Option<&str>,
) -> Result<TimeSession, AppError> {
    let existing = get_session(conn, id)?;
    client_service::get_client(conn, client_id)?;
    if stopped_at <= started_at {
        return Err(AppError::invalid_session());
    }

    let actual_duration_seconds = (stopped_at - started_at).num_seconds();
    let rounded_duration_minutes = round_up_from_seconds(actual_duration_seconds);
    let note = normalize_note(note);

    conn.execute(
        "UPDATE time_sessions
         SET client_id = ?1,
             started_at = ?2,
             stopped_at = ?3,
             actual_duration_seconds = ?4,
             rounded_duration_minutes = ?5,
             note = ?6
         WHERE id = ?7",
        params![
            client_id,
            started_at.to_rfc3339(),
            stopped_at.to_rfc3339(),
            actual_duration_seconds,
            rounded_duration_minutes,
            note,
            existing.id,
        ],
    )?;

    get_session(conn, id)
}

pub fn delete_session(conn: &Connection, id: i64) -> Result<(), AppError> {
    let changed = conn.execute("DELETE FROM time_sessions WHERE id = ?1", [id])?;
    if changed == 0 {
        return Err(AppError::not_found("That time session could not be found."));
    }
    Ok(())
}

pub fn map_session(row: &rusqlite::Row<'_>) -> rusqlite::Result<TimeSession> {
    Ok(TimeSession {
        id: row.get(0)?,
        client_id: row.get(1)?,
        client_name: row.get(2)?,
        started_at: row.get(3)?,
        stopped_at: row.get(4)?,
        actual_duration_seconds: row.get(5)?,
        rounded_duration_minutes: row.get(6)?,
        note: row.get(7)?,
        created_at: row.get(8)?,
    })
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
    use chrono::FixedOffset;

    fn setup() -> Connection {
        db::open_in_memory().unwrap()
    }

    fn tz() -> FixedOffset {
        FixedOffset::west_opt(4 * 3600).unwrap()
    }

    fn at(stamp: &str) -> DateTime<Utc> {
        parse_rfc3339(stamp).unwrap()
    }

    #[test]
    fn lists_and_filters_sessions() {
        let conn = setup();
        let acme = client_service::create_client(&conn, "Acme", None).unwrap();
        let globex = client_service::create_client(&conn, "Globex", None).unwrap();
        insert_session(&conn, acme.id, at("2026-09-03T09:02:00-04:00"), at("2026-09-03T10:11:00-04:00"), None).unwrap();
        insert_session(&conn, globex.id, at("2026-09-03T10:30:00-04:00"), at("2026-09-03T11:02:00-04:00"), None).unwrap();
        insert_session(&conn, acme.id, at("2026-09-04T13:00:00-04:00"), at("2026-09-04T13:20:00-04:00"), None).unwrap();

        let sept3 = timeutil::parse_date("2026-09-03").unwrap();
        let all = get_sessions(&conn, None, Some(sept3), Some(sept3), &tz()).unwrap();
        assert_eq!(all.len(), 2);

        let acme_only = get_sessions(&conn, Some(acme.id), Some(sept3), Some(sept3), &tz()).unwrap();
        assert_eq!(acme_only.len(), 1);
        assert_eq!(acme_only[0].client_id, acme.id);
    }

    #[test]
    fn rejects_inverted_date_range() {
        let conn = setup();
        let from = timeutil::parse_date("2026-09-04").unwrap();
        let to = timeutil::parse_date("2026-09-03").unwrap();
        let error = get_sessions(&conn, None, Some(from), Some(to), &tz()).unwrap_err();
        assert_eq!(error.code, "INVALID_DATE_RANGE");
    }

    #[test]
    fn edit_recalculates_durations() {
        let conn = setup();
        let acme = client_service::create_client(&conn, "Acme", None).unwrap();
        let created = insert_session(
            &conn,
            acme.id,
            at("2026-09-03T09:00:00-04:00"),
            at("2026-09-03T09:16:00-04:00"),
            None,
        )
        .unwrap();
        assert_eq!(created.rounded_duration_minutes, 30);

        let updated = update_session(
            &conn,
            created.id,
            acme.id,
            at("2026-09-03T09:00:00-04:00"),
            at("2026-09-03T09:14:00-04:00"),
            Some("revised"),
        )
        .unwrap();
        assert_eq!(updated.actual_duration_seconds, 14 * 60);
        assert_eq!(updated.rounded_duration_minutes, 15);
        assert_eq!(updated.note.as_deref(), Some("revised"));
    }

    #[test]
    fn rejects_end_before_start() {
        let conn = setup();
        let acme = client_service::create_client(&conn, "Acme", None).unwrap();
        let error = insert_session(
            &conn,
            acme.id,
            at("2026-09-03T10:00:00-04:00"),
            at("2026-09-03T09:00:00-04:00"),
            None,
        )
        .unwrap_err();
        assert_eq!(error.code, "INVALID_SESSION");
    }

    #[test]
    fn deletes_session() {
        let conn = setup();
        let acme = client_service::create_client(&conn, "Acme", None).unwrap();
        let created = insert_session(
            &conn,
            acme.id,
            at("2026-09-03T09:00:00-04:00"),
            at("2026-09-03T09:20:00-04:00"),
            None,
        )
        .unwrap();
        delete_session(&conn, created.id).unwrap();
        assert!(get_session(&conn, created.id).is_err());
    }

    #[test]
    fn midnight_crossing_stays_on_start_date() {
        let conn = setup();
        let acme = client_service::create_client(&conn, "Acme", None).unwrap();
        insert_session(
            &conn,
            acme.id,
            at("2026-09-03T23:00:00-04:00"),
            at("2026-09-04T01:00:00-04:00"),
            None,
        )
        .unwrap();

        let sept3 = timeutil::parse_date("2026-09-03").unwrap();
        let sept4 = timeutil::parse_date("2026-09-04").unwrap();
        assert_eq!(get_sessions(&conn, None, Some(sept3), Some(sept3), &tz()).unwrap().len(), 1);
        assert_eq!(get_sessions(&conn, None, Some(sept4), Some(sept4), &tz()).unwrap().len(), 0);
    }
}
