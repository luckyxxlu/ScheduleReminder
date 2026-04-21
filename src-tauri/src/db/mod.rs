pub mod config;
pub mod migration;
pub mod reminder_template_repository;

#[cfg(test)]
mod tests {
    use mysql::prelude::Queryable;

    use super::config::{database_url_from_env, load_db_config, DbConfigError};
    use super::migration::{
        create_pool, initialize_mysql, migration_statements, run_migrations, validate_database_url,
        MigrationError,
    };

    #[test]
    fn reads_mysql_database_url_from_env() {
        unsafe {
            std::env::set_var("DATABASE_URL", "mysql://root:password@127.0.0.1:3306/schedule_reminder");
        }

        let actual = database_url_from_env().expect("DATABASE_URL should be readable");

        assert_eq!(
            actual,
            "mysql://root:password@127.0.0.1:3306/schedule_reminder"
        );

        unsafe {
            std::env::remove_var("DATABASE_URL");
        }
    }

    #[test]
    fn returns_error_when_database_url_is_missing() {
        unsafe {
            std::env::remove_var("DATABASE_URL");
        }

        let error = database_url_from_env().expect_err("missing DATABASE_URL should fail");

        assert_eq!(error, DbConfigError::MissingDatabaseUrl);
    }

    #[test]
    fn loads_database_config_from_env() {
        unsafe {
            std::env::set_var("DATABASE_URL", "mysql://root:password@127.0.0.1:3306/schedule_reminder");
        }

        let config = load_db_config().expect("config should load");

        assert_eq!(
            config.database_url,
            "mysql://root:password@127.0.0.1:3306/schedule_reminder"
        );

        unsafe {
            std::env::remove_var("DATABASE_URL");
        }
    }

    #[test]
    fn rejects_non_mysql_database_urls() {
        unsafe {
            std::env::set_var("DATABASE_URL", "sqlite://local.db");
        }

        let error = database_url_from_env().expect_err("non-mysql scheme should fail");

        assert_eq!(error, DbConfigError::UnsupportedScheme);

        unsafe {
            std::env::remove_var("DATABASE_URL");
        }
    }

    #[test]
    fn validates_mysql_database_url() {
        let result = validate_database_url("mysql://root:password@127.0.0.1:3306/schedule_reminder");

        assert!(result.is_ok());
    }

    #[test]
    fn rejects_mysql_url_without_database_name() {
        let result = validate_database_url("mysql://root:password@127.0.0.1:3306");

        assert!(matches!(result, Err(DbConfigError::MissingDatabaseName)));
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
        let result = create_pool("sqlite://local.db");

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
    fn integration_runs_mysql_migrations_when_database_url_is_provided() {
        let database_url = match std::env::var("TEST_DATABASE_URL") {
            Ok(value) => value,
            Err(_) => return,
        };

        let database_name = format!(
            "schedule_reminder_test_{}",
            std::process::id()
        );

        let admin_url = database_url.replace("/schedule_reminder_test", "/mysql");

        let admin_pool = create_pool(&admin_url).expect("admin pool should be creatable");
        {
            let mut admin_conn = admin_pool
                .get_conn()
                .expect("admin connection should be available");
            admin_conn
                .query_drop(format!("CREATE DATABASE IF NOT EXISTS {database_name}"))
                .expect("test database should be creatable");
        }

        let app_url = admin_url.replace("/mysql", &format!("/{database_name}"));
        let pool = initialize_mysql(&app_url).expect("database should initialize");
        run_migrations(&pool).expect("migrations should stay idempotent");

        {
            let mut conn = pool.get_conn().expect("connection should be available");
            conn.query_drop("INSERT INTO reminder_templates (id, title, category, event_type, event_payload_json, repeat_rule_json, default_grace_minutes, notify_sound, note, enabled, created_at, updated_at) VALUES ('tpl_1', '喝水', 'health', 'text', JSON_OBJECT('message', '喝水时间到了'), JSON_OBJECT('type', 'daily', 'interval', 1), 10, 1, NULL, 1, NOW(3), NOW(3))")
                .expect("template insert should succeed");

            conn.query_drop("INSERT INTO reminder_occurrences (id, template_id, scheduled_at, grace_deadline_at, snoozed_until, status, handled_at, created_at, updated_at) VALUES ('occ_1', 'tpl_1', NOW(3), NOW(3), NULL, 'pending', NULL, NOW(3), NOW(3))")
                .expect("occurrence insert should succeed");

            let duplicate = conn.query_drop("INSERT INTO reminder_occurrences (id, template_id, scheduled_at, grace_deadline_at, snoozed_until, status, handled_at, created_at, updated_at) SELECT 'occ_2', template_id, scheduled_at, grace_deadline_at, NULL, 'pending', NULL, NOW(3), NOW(3) FROM reminder_occurrences WHERE id = 'occ_1'");

            assert!(duplicate.is_err());
        }

        {
            let mut admin_conn = admin_pool
                .get_conn()
                .expect("admin connection should still be available");
            admin_conn
                .query_drop(format!("DROP DATABASE IF EXISTS {database_name}"))
                .expect("test database should be droppable");
        }
    }
}
