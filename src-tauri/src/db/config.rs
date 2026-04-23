#[derive(Debug, PartialEq, Eq)]
pub enum DbConfigError {
    UnsupportedScheme,
    MissingDatabasePath,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbConfig {
    pub database_url: String,
    pub database_path: String,
}

pub fn database_url_from_env() -> Result<String, DbConfigError> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://schedule-reminder.db".to_string());

    super::migration::validate_database_url(&database_url)?;

    Ok(database_url)
}

pub fn load_db_config() -> Result<DbConfig, DbConfigError> {
    let database_url = database_url_from_env()?;

    Ok(DbConfig {
        database_path: extract_database_path(&database_url)?,
        database_url,
    })
}

pub fn extract_database_path(database_url: &str) -> Result<String, DbConfigError> {
    let path = database_url
        .strip_prefix("sqlite://")
        .ok_or(DbConfigError::UnsupportedScheme)?
        .trim();

    if path.is_empty() {
        return Err(DbConfigError::MissingDatabasePath);
    }

    Ok(path.to_string())
}
