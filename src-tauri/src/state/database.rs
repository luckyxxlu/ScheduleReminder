use crate::db::migration::DbPool;

#[derive(Clone)]
pub struct DatabaseState {
    pub pool: DbPool,
    pub database_path: Option<String>,
}

impl DatabaseState {
    pub fn new(pool: DbPool) -> Self {
        Self {
            pool,
            database_path: None,
        }
    }

    pub fn with_database_path(pool: DbPool, database_path: String) -> Self {
        Self {
            pool,
            database_path: Some(database_path),
        }
    }
}
