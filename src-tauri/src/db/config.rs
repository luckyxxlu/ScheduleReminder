#[derive(Debug, PartialEq, Eq)]
pub enum DbConfigError {
    MissingDatabaseUrl,
    UnsupportedScheme,
    MissingDatabaseName,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbConfig {
    pub database_url: String,
}

pub fn database_url_from_env() -> Result<String, DbConfigError> {
    let database_url = std::env::var("DATABASE_URL").map_err(|_| DbConfigError::MissingDatabaseUrl)?;

    super::migration::validate_database_url(&database_url)?;

    Ok(database_url)
}

pub fn load_db_config() -> Result<DbConfig, DbConfigError> {
    Ok(DbConfig {
        database_url: database_url_from_env()?,
    })
}
