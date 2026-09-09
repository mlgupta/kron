use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};

use crate::error::AppError;
use crate::models::Client;

pub fn create_client(
    conn: &Connection,
    name: &str,
    description: Option<&str>,
) -> Result<Client, AppError> {
    let name = normalize_name(name)?;
    let description = normalize_description(description);
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO clients (name, description, active, created_at, updated_at)
         VALUES (?1, ?2, 1, ?3, ?3)",
        params![name, description, now],
    )?;

    get_client(conn, conn.last_insert_rowid())
}

pub fn get_clients(conn: &Connection) -> Result<Vec<Client>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, active, created_at, updated_at
         FROM clients
         ORDER BY name COLLATE NOCASE ASC",
    )?;
    let clients = stmt
        .query_map([], map_client)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(clients)
}

pub fn get_client(conn: &Connection, id: i64) -> Result<Client, AppError> {
    conn.query_row(
        "SELECT id, name, description, active, created_at, updated_at
         FROM clients
         WHERE id = ?1",
        [id],
        map_client,
    )
    .optional()?
    .ok_or_else(|| AppError::not_found("That client could not be found."))
}

pub fn update_client(
    conn: &Connection,
    id: i64,
    name: &str,
    description: Option<&str>,
) -> Result<Client, AppError> {
    let name = normalize_name(name)?;
    let description = normalize_description(description);
    let now = Utc::now().to_rfc3339();

    let existing = get_client(conn, id)?;
    if names_equal(&existing.name, &name)
        && existing.description.as_deref() == description.as_deref()
    {
        return Ok(existing);
    }

    let changed = conn.execute(
        "UPDATE clients
         SET name = ?1, description = ?2, updated_at = ?3
         WHERE id = ?4",
        params![name, description, now, id],
    )?;

    if changed == 0 {
        return Err(AppError::not_found("That client could not be found."));
    }

    get_client(conn, id)
}

pub fn set_client_active(conn: &Connection, id: i64, active: bool) -> Result<Client, AppError> {
    let now = Utc::now().to_rfc3339();
    let changed = conn.execute(
        "UPDATE clients SET active = ?1, updated_at = ?2 WHERE id = ?3",
        params![active as i64, now, id],
    )?;

    if changed == 0 {
        return Err(AppError::not_found("That client could not be found."));
    }

    get_client(conn, id)
}

pub fn delete_client(conn: &Connection, id: i64, force: bool) -> Result<(), AppError> {
    let _existing = get_client(conn, id)?;
    let session_count = count_sessions(conn, id)?;

    if session_count > 0 && !force {
        return Err(AppError::client_has_sessions());
    }

    let tx = conn.unchecked_transaction()?;
    if session_count > 0 {
        tx.execute("DELETE FROM time_sessions WHERE client_id = ?1", [id])?;
    }
    tx.execute("DELETE FROM active_timer WHERE client_id = ?1", [id])?;
    tx.execute("DELETE FROM clients WHERE id = ?1", [id])?;
    tx.commit()?;
    Ok(())
}

fn count_sessions(conn: &Connection, client_id: i64) -> Result<i64, AppError> {
    let count = conn.query_row(
        "SELECT COUNT(*) FROM time_sessions WHERE client_id = ?1",
        [client_id],
        |row| row.get(0),
    )?;
    Ok(count)
}

fn map_client(row: &rusqlite::Row<'_>) -> rusqlite::Result<Client> {
    let active: i64 = row.get(3)?;
    Ok(Client {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        active: active != 0,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

fn normalize_name(name: &str) -> Result<String, AppError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AppError::validation("Client name is required."));
    }
    if trimmed.chars().count() > 120 {
        return Err(AppError::validation(
            "Client name must be 120 characters or fewer.",
        ));
    }
    Ok(trimmed.to_string())
}

fn normalize_description(description: Option<&str>) -> Option<String> {
    description
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn names_equal(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn setup() -> Connection {
        db::open_in_memory().expect("in-memory database")
    }

    fn insert_session(conn: &Connection, client_id: i64) {
        conn.execute(
            "INSERT INTO time_sessions (
                client_id, started_at, stopped_at,
                actual_duration_seconds, rounded_duration_minutes, created_at
             ) VALUES (?1, ?2, ?2, 60, 15, ?2)",
            params![client_id, Utc::now().to_rfc3339()],
        )
        .unwrap();
    }

    #[test]
    fn creates_and_reads_client() {
        let conn = setup();
        let created = create_client(&conn, "  Acme  ", Some(" Consulting ")).unwrap();

        assert_eq!(created.name, "Acme");
        assert_eq!(created.description.as_deref(), Some("Consulting"));
        assert!(created.active);

        let fetched = get_client(&conn, created.id).unwrap();
        assert_eq!(fetched, created);
    }

    #[test]
    fn rejects_empty_name() {
        let conn = setup();
        let error = create_client(&conn, "   ", None).unwrap_err();
        assert_eq!(error.code, "VALIDATION");
    }

    #[test]
    fn rejects_duplicate_names_case_insensitively() {
        let conn = setup();
        create_client(&conn, "Acme", None).unwrap();
        let error = create_client(&conn, "acme", Some("Other")).unwrap_err();
        assert_eq!(error.code, "DUPLICATE_NAME");
    }

    #[test]
    fn lists_clients_sorted_by_name() {
        let conn = setup();
        create_client(&conn, "Zed", None).unwrap();
        create_client(&conn, "alpha", None).unwrap();

        let names: Vec<String> = get_clients(&conn)
            .unwrap()
            .into_iter()
            .map(|client| client.name)
            .collect();
        assert_eq!(names, vec!["alpha".to_string(), "Zed".to_string()]);
    }

    #[test]
    fn updates_client() {
        let conn = setup();
        let created = create_client(&conn, "Acme", None).unwrap();
        let updated = update_client(&conn, created.id, "Acme Corp", Some("Legal")).unwrap();

        assert_eq!(updated.name, "Acme Corp");
        assert_eq!(updated.description.as_deref(), Some("Legal"));
        assert_ne!(updated.updated_at, created.updated_at);
    }

    #[test]
    fn update_rejects_duplicate_name() {
        let conn = setup();
        create_client(&conn, "Acme", None).unwrap();
        let other = create_client(&conn, "Globex", None).unwrap();
        let error = update_client(&conn, other.id, "acme", None).unwrap_err();
        assert_eq!(error.code, "DUPLICATE_NAME");
    }

    #[test]
    fn activates_and_deactivates_client() {
        let conn = setup();
        let created = create_client(&conn, "Acme", None).unwrap();
        let inactive = set_client_active(&conn, created.id, false).unwrap();
        assert!(!inactive.active);
        let active = set_client_active(&conn, created.id, true).unwrap();
        assert!(active.active);
    }

    #[test]
    fn deletes_client_without_sessions() {
        let conn = setup();
        let created = create_client(&conn, "Acme", None).unwrap();
        delete_client(&conn, created.id, false).unwrap();
        let error = get_client(&conn, created.id).unwrap_err();
        assert_eq!(error.code, "NOT_FOUND");
    }

    #[test]
    fn refuses_delete_when_sessions_exist() {
        let conn = setup();
        let created = create_client(&conn, "Acme", None).unwrap();
        insert_session(&conn, created.id);

        let error = delete_client(&conn, created.id, false).unwrap_err();
        assert_eq!(error.code, "CLIENT_HAS_SESSIONS");
        assert!(get_client(&conn, created.id).is_ok());
    }

    #[test]
    fn force_deletes_client_and_sessions() {
        let conn = setup();
        let created = create_client(&conn, "Acme", None).unwrap();
        insert_session(&conn, created.id);

        delete_client(&conn, created.id, true).unwrap();
        assert!(get_client(&conn, created.id).is_err());

        let remaining: i64 = conn
            .query_row("SELECT COUNT(*) FROM time_sessions", [], |row| row.get(0))
            .unwrap();
        assert_eq!(remaining, 0);
    }

    #[test]
    fn missing_client_returns_not_found() {
        let conn = setup();
        let error = get_client(&conn, 99).unwrap_err();
        assert_eq!(error.code, "NOT_FOUND");
    }
}
