#[derive(Debug, PartialEq, Eq)]
pub enum DbConfigError {
    MissingDatabaseUrl,
    UnsupportedScheme,
    MissingDatabaseName,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbConfig {
    pub database_url: String,
    pub database_name: String,
}

pub fn database_url_from_env() -> Result<String, DbConfigError> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mysql://root:root@127.0.0.1:3306/schedule_reminder".to_string());

    super::migration::validate_database_url(&database_url)?;

    Ok(database_url)
}

pub fn load_db_config() -> Result<DbConfig, DbConfigError> {
    let database_url = database_url_from_env()?;

    Ok(DbConfig {
        database_name: extract_database_name(&database_url)?,
        database_url,
    })
}

fn extract_database_name(database_url: &str) -> Result<String, DbConfigError> {
    let without_scheme = database_url
        .strip_prefix("mysql://")
        .ok_or(DbConfigError::UnsupportedScheme)?;

    let database_name = without_scheme
        .rsplit('/')
        .next()
        .filter(|value| !value.is_empty())
        .ok_or(DbConfigError::MissingDatabaseName)?;

    Ok(database_name.to_string())
}
