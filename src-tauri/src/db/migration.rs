use super::config::DbConfigError;
use mysql::{prelude::Queryable, Opts, Pool};

#[derive(Debug, PartialEq, Eq)]
pub enum MigrationError {
    InvalidConfig(DbConfigError),
    MysqlInitFailed,
    MysqlConnectionFailed,
    StatementExecutionFailed,
}

pub fn validate_database_url(database_url: &str) -> Result<(), DbConfigError> {
    if !database_url.starts_with("mysql://") {
        return Err(DbConfigError::UnsupportedScheme);
    }

    let without_scheme = &database_url[8..];

    if !without_scheme.contains('/') || without_scheme.ends_with('/') {
        return Err(DbConfigError::MissingDatabaseName);
    }

    Ok(())
}

pub fn create_pool(database_url: &str) -> Result<Pool, MigrationError> {
    validate_database_url(database_url).map_err(MigrationError::InvalidConfig)?;

    let opts = Opts::from_url(database_url).map_err(|_| MigrationError::MysqlInitFailed)?;

    Pool::new(opts).map_err(|_| MigrationError::MysqlConnectionFailed)
}

pub fn create_database_if_missing(database_url: &str) -> Result<(), MigrationError> {
    validate_database_url(database_url).map_err(MigrationError::InvalidConfig)?;

    let (admin_url, database_name) = split_admin_url(database_url)
        .map_err(MigrationError::InvalidConfig)?;

    let opts = Opts::from_url(&admin_url).map_err(|_| MigrationError::MysqlInitFailed)?;
    let pool = Pool::new(opts).map_err(|_| MigrationError::MysqlConnectionFailed)?;
    let mut connection = pool
        .get_conn()
        .map_err(|_| MigrationError::MysqlConnectionFailed)?;

    connection
        .query_drop(format!(
            "CREATE DATABASE IF NOT EXISTS `{}` CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci",
            database_name
        ))
        .map_err(|_| MigrationError::StatementExecutionFailed)?;

    Ok(())
}

pub fn run_migrations(pool: &Pool) -> Result<(), MigrationError> {
    let mut connection = pool
        .get_conn()
        .map_err(|_| MigrationError::MysqlConnectionFailed)?;

    for statement in migration_statements() {
        connection
            .query_drop(statement)
            .map_err(|_| MigrationError::StatementExecutionFailed)?;
    }

        connection
            .query_drop(
                "INSERT INTO app_settings (id, default_grace_minutes, startup_with_windows, tray_enabled, close_to_tray_on_close, theme, quiet_hours_enabled, quiet_hours_start, quiet_hours_end, updated_at) VALUES (1, 10, 0, 1, 1, 'system', 0, NULL, NULL, NOW(3)) ON DUPLICATE KEY UPDATE updated_at = VALUES(updated_at)"
        )
        .map_err(|_| MigrationError::StatementExecutionFailed)?;

    Ok(())
}

pub fn initialize_mysql(database_url: &str) -> Result<Pool, MigrationError> {
    create_database_if_missing(database_url)?;
    let pool = create_pool(database_url)?;
    run_migrations(&pool)?;
    Ok(pool)
}

fn split_admin_url(database_url: &str) -> Result<(String, String), DbConfigError> {
    validate_database_url(database_url)?;

    let without_scheme = database_url
        .strip_prefix("mysql://")
        .ok_or(DbConfigError::UnsupportedScheme)?;
    let (authority, database_name) = without_scheme
        .rsplit_once('/')
        .ok_or(DbConfigError::MissingDatabaseName)?;

    if database_name.is_empty() {
        return Err(DbConfigError::MissingDatabaseName);
    }

    Ok((format!("mysql://{authority}/mysql"), database_name.to_string()))
}

pub fn migration_statements() -> Vec<&'static str> {
    vec![
        "CREATE TABLE IF NOT EXISTS reminder_templates (id VARCHAR(64) PRIMARY KEY NOT NULL, title VARCHAR(255) NOT NULL, category VARCHAR(64) NULL, event_type VARCHAR(32) NOT NULL, event_payload_json JSON NOT NULL, repeat_rule_json JSON NOT NULL, default_grace_minutes INTEGER NOT NULL, notify_sound TINYINT(1) NOT NULL DEFAULT 1, note TEXT NULL, enabled TINYINT(1) NOT NULL DEFAULT 1, created_at DATETIME(3) NOT NULL, updated_at DATETIME(3) NOT NULL) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;",
        "CREATE TABLE IF NOT EXISTS reminder_occurrences (id VARCHAR(64) PRIMARY KEY NOT NULL, template_id VARCHAR(64) NOT NULL, scheduled_at DATETIME(3) NOT NULL, grace_deadline_at DATETIME(3) NOT NULL, snoozed_until DATETIME(3) NULL, status VARCHAR(32) NOT NULL, handled_at DATETIME(3) NULL, created_at DATETIME(3) NOT NULL, updated_at DATETIME(3) NOT NULL, CONSTRAINT fk_occurrence_template FOREIGN KEY(template_id) REFERENCES reminder_templates(id), UNIQUE KEY uq_template_scheduled (template_id, scheduled_at)) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;",
        "CREATE TABLE IF NOT EXISTS reminder_action_logs (id VARCHAR(64) PRIMARY KEY NOT NULL, occurrence_id VARCHAR(64) NOT NULL, action VARCHAR(64) NOT NULL, action_at DATETIME(3) NOT NULL, payload_json JSON NULL, CONSTRAINT fk_action_log_occurrence FOREIGN KEY(occurrence_id) REFERENCES reminder_occurrences(id)) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;",
        "CREATE TABLE IF NOT EXISTS app_settings (id INTEGER PRIMARY KEY NOT NULL, default_grace_minutes INTEGER NOT NULL, startup_with_windows TINYINT(1) NOT NULL DEFAULT 0, tray_enabled TINYINT(1) NOT NULL DEFAULT 1, close_to_tray_on_close TINYINT(1) NOT NULL DEFAULT 1, theme VARCHAR(32) NOT NULL DEFAULT 'system', quiet_hours_enabled TINYINT(1) NOT NULL DEFAULT 0, quiet_hours_start VARCHAR(8) NULL, quiet_hours_end VARCHAR(8) NULL, updated_at DATETIME(3) NOT NULL) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;",
    ]
}
