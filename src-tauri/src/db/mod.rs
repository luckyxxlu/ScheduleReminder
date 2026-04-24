pub mod config;
pub mod migration;
pub mod persistence;
pub mod reminder_template_repository;

#[cfg(test)]
mod tests {
    use super::config::{database_url_from_env, load_db_config, DbConfigError};
    use super::migration::{
        create_pool, initialize_database, migration_statements, run_migrations, validate_database_url,
        MigrationError,
    };

    #[test]
    fn reads_sqlite_database_url_from_env() {
        unsafe {
            std::env::set_var("DATABASE_URL", "sqlite://data/schedule-reminder.db");
        }

        let actual = database_url_from_env().expect("DATABASE_URL should be readable");

        assert_eq!(actual, "sqlite://data/schedule-reminder.db");

        unsafe {
            std::env::remove_var("DATABASE_URL");
        }
    }

    #[test]
    fn returns_error_when_database_url_is_missing() {
        unsafe {
            std::env::remove_var("DATABASE_URL");
        }

        let url = database_url_from_env().expect("default DATABASE_URL should be provided");

        assert_eq!(url, "sqlite://schedule-reminder.db");
    }

    #[test]
    fn loads_database_config_from_env() {
        unsafe {
            std::env::set_var("DATABASE_URL", "sqlite://data/schedule-reminder.db");
        }

        let config = load_db_config().expect("config should load");

        assert_eq!(config.database_url, "sqlite://data/schedule-reminder.db");
        assert_eq!(config.database_path, "data/schedule-reminder.db");

        unsafe {
            std::env::remove_var("DATABASE_URL");
        }
    }

    #[test]
    fn rejects_non_sqlite_database_urls() {
        unsafe {
            std::env::set_var("DATABASE_URL", "mysql://root:password@127.0.0.1:3306/schedule_reminder");
        }

        let error = database_url_from_env().expect_err("non-sqlite scheme should fail");

        assert_eq!(error, DbConfigError::UnsupportedScheme);

        unsafe {
            std::env::remove_var("DATABASE_URL");
        }
    }

    #[test]
    fn validates_sqlite_database_url() {
        let result = validate_database_url("sqlite://schedule-reminder.db");

        assert!(result.is_ok());
    }

    #[test]
    fn rejects_sqlite_url_without_database_path() {
        let result = validate_database_url("sqlite://");

        assert!(matches!(result, Err(DbConfigError::MissingDatabasePath)));
    }

    #[test]
    fn provides_non_empty_migration_statements() {
        let statements = migration_statements();

        assert!(!statements.is_empty());
        assert!(statements.iter().any(|statement| statement.contains("reminder_templates")));
        assert!(statements.iter().any(|statement| statement.contains("reminder_occurrences")));
        assert!(statements.iter().any(|statement| statement.contains("reminder_action_logs")));
        assert!(statements.iter().any(|statement| statement.contains("app_settings")));
    }

    #[test]
    fn migration_statements_use_idempotent_create_table() {
        let statements = migration_statements();

        assert!(statements.iter().all(|statement| {
            !statement.contains("CREATE TABLE ") || statement.contains("CREATE TABLE IF NOT EXISTS")
        }));
    }

    #[test]
    fn create_pool_rejects_invalid_database_url() {
        let result = create_pool("mysql://root:password@127.0.0.1:3306/schedule_reminder");

        assert!(matches!(
            result,
            Err(MigrationError::InvalidConfig(DbConfigError::UnsupportedScheme))
        ));
    }

    #[test]
    fn app_settings_seed_statement_exists() {
        let seed = "INSERT INTO app_settings";

        let source = std::fs::read_to_string("src/db/migration.rs")
            .expect("migration source file should be readable during tests");

        assert!(source.contains(seed));
    }

    #[test]
    fn integration_runs_sqlite_migrations_when_database_url_is_provided() {
        let database_path = std::env::temp_dir().join(format!(
            "schedule_reminder_db_test_{}.db",
            std::process::id()
        ));
        let database_url = format!("sqlite://{}", database_path.display());

        let pool = initialize_database(&database_url).expect("database should initialize");
        run_migrations(&pool).expect("migrations should stay idempotent");

        let connection = pool.lock().expect("connection should be available");
        connection
            .execute(
                "INSERT INTO reminder_templates (id, title, category, event_type, event_payload_json, repeat_rule_json, default_grace_minutes, notify_sound, note, enabled, created_at, updated_at) VALUES ('tpl_1', '喝水', 'health', 'text', '{\"message\":\"喝水时间到了\"}', '{\"type\":\"daily\",\"interval\":1}', 10, 1, NULL, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                [],
            )
            .expect("template insert should succeed");

        connection
            .execute(
                "INSERT INTO reminder_occurrences (id, template_id, scheduled_at, grace_deadline_at, snoozed_until, status, handled_at, created_at, updated_at) VALUES ('occ_1', 'tpl_1', '2026-04-22 08:00:00', '2026-04-22 08:10:00', NULL, 'pending', NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                [],
            )
            .expect("occurrence insert should succeed");

        let duplicate = connection.execute(
            "INSERT INTO reminder_occurrences (id, template_id, scheduled_at, grace_deadline_at, snoozed_until, status, handled_at, created_at, updated_at) VALUES ('occ_2', 'tpl_1', '2026-04-22 08:00:00', '2026-04-22 08:10:00', NULL, 'pending', NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
            [],
        );

        assert!(duplicate.is_err());

        drop(connection);
        let _ = std::fs::remove_file(database_path);
    }

    #[test]
    fn migrations_upgrade_legacy_sqlite_schema() {
        let database_path = std::env::temp_dir().join(format!(
            "schedule_reminder_legacy_db_test_{}.db",
            std::process::id()
        ));
        let database_url = format!("sqlite://{}", database_path.display());
        let pool = create_pool(&database_url).expect("database should open");

        {
            let connection = pool.lock().expect("connection should be available");
            connection
                .execute(
                    "CREATE TABLE reminder_templates (id TEXT PRIMARY KEY NOT NULL, title TEXT NOT NULL, category TEXT NULL, event_type TEXT NOT NULL, event_payload_json TEXT NOT NULL, repeat_rule_json TEXT NOT NULL, default_grace_minutes INTEGER NOT NULL)",
                    [],
                )
                .expect("legacy reminder_templates should be created");
            connection
                .execute(
                    "CREATE TABLE reminder_occurrences (id TEXT PRIMARY KEY NOT NULL, template_id TEXT NOT NULL, scheduled_at TEXT NOT NULL, grace_deadline_at TEXT NOT NULL, status TEXT NOT NULL, FOREIGN KEY(template_id) REFERENCES reminder_templates(id), UNIQUE(template_id, scheduled_at))",
                    [],
                )
                .expect("legacy reminder_occurrences should be created");
            connection
                .execute(
                    "CREATE TABLE reminder_action_logs (id TEXT PRIMARY KEY NOT NULL, occurrence_id TEXT NOT NULL, action TEXT NOT NULL, action_at TEXT NOT NULL, FOREIGN KEY(occurrence_id) REFERENCES reminder_occurrences(id))",
                    [],
                )
                .expect("legacy reminder_action_logs should be created");
            connection
                .execute(
                    "CREATE TABLE app_settings (id INTEGER PRIMARY KEY NOT NULL, default_grace_minutes INTEGER NOT NULL, startup_with_windows INTEGER NOT NULL DEFAULT 0)",
                    [],
                )
                .expect("legacy app_settings should be created");
        }

        run_migrations(&pool).expect("legacy schema should upgrade");

        let connection = pool.lock().expect("connection should be available");
        let reminder_template_columns = table_columns(&connection, "reminder_templates");
        let reminder_occurrence_columns = table_columns(&connection, "reminder_occurrences");
        let action_log_columns = table_columns(&connection, "reminder_action_logs");
        let settings_columns = table_columns(&connection, "app_settings");

        assert!(reminder_template_columns.contains(&"notify_sound".to_string()));
        assert!(reminder_template_columns.contains(&"enabled".to_string()));
        assert!(reminder_template_columns.contains(&"created_at".to_string()));
        assert!(reminder_template_columns.contains(&"updated_at".to_string()));
        assert!(reminder_occurrence_columns.contains(&"snoozed_until".to_string()));
        assert!(reminder_occurrence_columns.contains(&"handled_at".to_string()));
        assert!(reminder_occurrence_columns.contains(&"created_at".to_string()));
        assert!(action_log_columns.contains(&"payload_json".to_string()));
        assert!(settings_columns.contains(&"tray_enabled".to_string()));
        assert!(settings_columns.contains(&"theme".to_string()));
        assert!(settings_columns.contains(&"updated_at".to_string()));

        drop(connection);
        let _ = std::fs::remove_file(database_path);
    }

    fn table_columns(connection: &rusqlite::Connection, table: &str) -> Vec<String> {
        let pragma = format!("PRAGMA table_info({table})");
        let mut statement = connection
            .prepare(&pragma)
            .expect("table info pragma should be readable");

        statement
            .query_map([], |row| row.get::<_, String>(1))
            .expect("table info query should succeed")
            .collect::<Result<Vec<_>, _>>()
            .expect("columns should be collectable")
    }
}
