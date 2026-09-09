use rusqlite::Connection;

use crate::error::{log_technical_error, AppError};

struct Migration {
    version: i64,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    sql: r#"
        CREATE TABLE clients (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            description TEXT,
            active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE UNIQUE INDEX idx_clients_name_unique
            ON clients(name COLLATE NOCASE);

        CREATE TABLE time_sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            client_id INTEGER NOT NULL,
            started_at TEXT NOT NULL,
            stopped_at TEXT NOT NULL,
            actual_duration_seconds INTEGER NOT NULL,
            rounded_duration_minutes INTEGER NOT NULL,
            note TEXT,
            created_at TEXT NOT NULL,

            FOREIGN KEY (client_id)
                REFERENCES clients(id)
        );

        CREATE TABLE active_timer (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            client_id INTEGER NOT NULL,
            started_at TEXT NOT NULL,
            note TEXT,

            FOREIGN KEY (client_id)
                REFERENCES clients(id)
        );
    "#,
}];

pub fn run(conn: &Connection) -> Result<(), AppError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL
        )",
        [],
    )?;

    let applied = current_version(conn)?;

    for migration in MIGRATIONS {
        if migration.version <= applied {
            continue;
        }

        let tx = conn.unchecked_transaction()?;
        if let Err(error) = tx.execute_batch(migration.sql) {
            log_technical_error("migration", &error);
            return Err(AppError::database());
        }

        tx.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, datetime('now'))",
            [migration.version],
        )?;
        tx.commit()?;
    }

    Ok(())
}

fn current_version(conn: &Connection) -> Result<i64, AppError> {
    let version = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    Ok(version)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn applies_initial_schema() {
        let conn = Connection::open_in_memory().unwrap();
        run(&conn).unwrap();

        let tables: Vec<String> = conn
            .prepare(
                "SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name",
            )
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(Result::ok)
            .collect();

        assert!(tables.contains(&"clients".to_string()));
        assert!(tables.contains(&"time_sessions".to_string()));
        assert!(tables.contains(&"active_timer".to_string()));
        assert_eq!(current_version(&conn).unwrap(), 1);
    }

    #[test]
    fn is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        run(&conn).unwrap();
        run(&conn).unwrap();
        assert_eq!(current_version(&conn).unwrap(), 1);
    }
}
