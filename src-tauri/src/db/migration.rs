use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use super::config::{extract_database_path, DbConfigError};

pub type DbPool = Arc<Mutex<Connection>>;

#[derive(Debug, PartialEq, Eq)]
pub enum MigrationError {
    InvalidConfig(DbConfigError),
    SqliteOpenFailed,
    StatementExecutionFailed,
}

pub fn validate_database_url(database_url: &str) -> Result<(), DbConfigError> {
    extract_database_path(database_url).map(|_| ())
}

pub fn create_pool(database_url: &str) -> Result<DbPool, MigrationError> {
    let database_path =
        extract_database_path(database_url).map_err(MigrationError::InvalidConfig)?;

    if let Some(parent) = Path::new(&database_path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|_| MigrationError::SqliteOpenFailed)?;
        }
    }

    let connection =
        Connection::open(database_path).map_err(|_| MigrationError::SqliteOpenFailed)?;
    connection
        .pragma_update(None, "foreign_keys", "ON")
        .map_err(|_| MigrationError::StatementExecutionFailed)?;

    Ok(Arc::new(Mutex::new(connection)))
}

pub fn run_migrations(pool: &DbPool) -> Result<(), MigrationError> {
    let connection = pool
        .lock()
        .map_err(|_| MigrationError::StatementExecutionFailed)?;

    for statement in migration_statements() {
        connection
            .execute(statement, [])
            .map_err(|_| MigrationError::StatementExecutionFailed)?;
    }

    ensure_legacy_columns(&connection)?;

    connection
        .execute(
            "INSERT INTO app_settings (id, default_grace_minutes, startup_with_windows, tray_enabled, close_to_tray_on_close, theme, quiet_hours_enabled, quiet_hours_start, quiet_hours_end, updated_at) VALUES (1, 10, 0, 1, 1, 'system', 0, NULL, NULL, CURRENT_TIMESTAMP) ON CONFLICT(id) DO NOTHING",
            [],
        )
        .map_err(|_| MigrationError::StatementExecutionFailed)?;

    Ok(())
}

pub fn initialize_database(database_url: &str) -> Result<DbPool, MigrationError> {
    let pool = create_pool(database_url)?;
    run_migrations(&pool)?;
    Ok(pool)
}

pub fn migration_statements() -> Vec<&'static str> {
    vec![
        "CREATE TABLE IF NOT EXISTS reminder_templates (id TEXT PRIMARY KEY NOT NULL, title TEXT NOT NULL, category TEXT NULL, event_type TEXT NOT NULL, event_payload_json TEXT NOT NULL, repeat_rule_json TEXT NOT NULL, default_grace_minutes INTEGER NOT NULL, notify_sound INTEGER NOT NULL DEFAULT 1, note TEXT NULL, enabled INTEGER NOT NULL DEFAULT 1, created_at TEXT NOT NULL, updated_at TEXT NOT NULL)",
        "CREATE TABLE IF NOT EXISTS reminder_occurrences (id TEXT PRIMARY KEY NOT NULL, template_id TEXT NOT NULL, scheduled_at TEXT NOT NULL, grace_deadline_at TEXT NOT NULL, snoozed_until TEXT NULL, status TEXT NOT NULL, handled_at TEXT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL, FOREIGN KEY(template_id) REFERENCES reminder_templates(id), UNIQUE(template_id, scheduled_at))",
        "CREATE TABLE IF NOT EXISTS reminder_action_logs (id TEXT PRIMARY KEY NOT NULL, occurrence_id TEXT NOT NULL, action TEXT NOT NULL, action_at TEXT NOT NULL, payload_json TEXT NULL, FOREIGN KEY(occurrence_id) REFERENCES reminder_occurrences(id))",
        "CREATE TABLE IF NOT EXISTS app_settings (id INTEGER PRIMARY KEY NOT NULL, default_grace_minutes INTEGER NOT NULL, startup_with_windows INTEGER NOT NULL DEFAULT 0, tray_enabled INTEGER NOT NULL DEFAULT 1, close_to_tray_on_close INTEGER NOT NULL DEFAULT 1, theme TEXT NOT NULL DEFAULT 'system', quiet_hours_enabled INTEGER NOT NULL DEFAULT 0, quiet_hours_start TEXT NULL, quiet_hours_end TEXT NULL, updated_at TEXT NOT NULL)",
    ]
}

fn ensure_legacy_columns(connection: &Connection) -> Result<(), MigrationError> {
    ensure_column(
        connection,
        "reminder_templates",
        "notify_sound",
        "ALTER TABLE reminder_templates ADD COLUMN notify_sound INTEGER NOT NULL DEFAULT 1",
        None,
    )?;
    ensure_column(
        connection,
        "reminder_templates",
        "note",
        "ALTER TABLE reminder_templates ADD COLUMN note TEXT NULL",
        None,
    )?;
    ensure_column(
        connection,
        "reminder_templates",
        "enabled",
        "ALTER TABLE reminder_templates ADD COLUMN enabled INTEGER NOT NULL DEFAULT 1",
        None,
    )?;
    ensure_column(
        connection,
        "reminder_templates",
        "created_at",
        "ALTER TABLE reminder_templates ADD COLUMN created_at TEXT NOT NULL DEFAULT ''",
        Some("UPDATE reminder_templates SET created_at = CURRENT_TIMESTAMP WHERE created_at = ''"),
    )?;
    ensure_column(
        connection,
        "reminder_templates",
        "updated_at",
        "ALTER TABLE reminder_templates ADD COLUMN updated_at TEXT NOT NULL DEFAULT ''",
        Some("UPDATE reminder_templates SET updated_at = CURRENT_TIMESTAMP WHERE updated_at = ''"),
    )?;

    ensure_column(
        connection,
        "reminder_occurrences",
        "snoozed_until",
        "ALTER TABLE reminder_occurrences ADD COLUMN snoozed_until TEXT NULL",
        None,
    )?;
    ensure_column(
        connection,
        "reminder_occurrences",
        "handled_at",
        "ALTER TABLE reminder_occurrences ADD COLUMN handled_at TEXT NULL",
        None,
    )?;
    ensure_column(
        connection,
        "reminder_occurrences",
        "created_at",
        "ALTER TABLE reminder_occurrences ADD COLUMN created_at TEXT NOT NULL DEFAULT ''",
        Some(
            "UPDATE reminder_occurrences SET created_at = CURRENT_TIMESTAMP WHERE created_at = ''",
        ),
    )?;
    ensure_column(
        connection,
        "reminder_occurrences",
        "updated_at",
        "ALTER TABLE reminder_occurrences ADD COLUMN updated_at TEXT NOT NULL DEFAULT ''",
        Some(
            "UPDATE reminder_occurrences SET updated_at = CURRENT_TIMESTAMP WHERE updated_at = ''",
        ),
    )?;

    ensure_column(
        connection,
        "reminder_action_logs",
        "payload_json",
        "ALTER TABLE reminder_action_logs ADD COLUMN payload_json TEXT NULL",
        None,
    )?;

    ensure_column(
        connection,
        "app_settings",
        "tray_enabled",
        "ALTER TABLE app_settings ADD COLUMN tray_enabled INTEGER NOT NULL DEFAULT 1",
        None,
    )?;
    ensure_column(
        connection,
        "app_settings",
        "close_to_tray_on_close",
        "ALTER TABLE app_settings ADD COLUMN close_to_tray_on_close INTEGER NOT NULL DEFAULT 1",
        None,
    )?;
    ensure_column(
        connection,
        "app_settings",
        "theme",
        "ALTER TABLE app_settings ADD COLUMN theme TEXT NOT NULL DEFAULT 'system'",
        None,
    )?;
    ensure_column(
        connection,
        "app_settings",
        "quiet_hours_enabled",
        "ALTER TABLE app_settings ADD COLUMN quiet_hours_enabled INTEGER NOT NULL DEFAULT 0",
        None,
    )?;
    ensure_column(
        connection,
        "app_settings",
        "quiet_hours_start",
        "ALTER TABLE app_settings ADD COLUMN quiet_hours_start TEXT NULL",
        None,
    )?;
    ensure_column(
        connection,
        "app_settings",
        "quiet_hours_end",
        "ALTER TABLE app_settings ADD COLUMN quiet_hours_end TEXT NULL",
        None,
    )?;
    ensure_column(
        connection,
        "app_settings",
        "updated_at",
        "ALTER TABLE app_settings ADD COLUMN updated_at TEXT NOT NULL DEFAULT ''",
        Some("UPDATE app_settings SET updated_at = CURRENT_TIMESTAMP WHERE updated_at = ''"),
    )?;

    Ok(())
}

/// Adds a missing column during startup migration and optionally backfills legacy rows.
fn ensure_column(
    connection: &Connection,
    table: &str,
    column: &str,
    add_statement: &str,
    backfill_statement: Option<&str>,
) -> Result<(), MigrationError> {
    if has_column(connection, table, column)? {
        return Ok(());
    }

    connection
        .execute(add_statement, [])
        .map_err(|_| MigrationError::StatementExecutionFailed)?;

    if let Some(statement) = backfill_statement {
        connection
            .execute(statement, [])
            .map_err(|_| MigrationError::StatementExecutionFailed)?;
    }

    Ok(())
}

fn has_column(connection: &Connection, table: &str, column: &str) -> Result<bool, MigrationError> {
    let pragma = format!("PRAGMA table_info({table})");
    let mut statement = connection
        .prepare(&pragma)
        .map_err(|_| MigrationError::StatementExecutionFailed)?;
    let columns = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|_| MigrationError::StatementExecutionFailed)?;

    for existing in columns {
        if existing.map_err(|_| MigrationError::StatementExecutionFailed)? == column {
            return Ok(true);
        }
    }

    Ok(false)
}
