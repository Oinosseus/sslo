use std::error::Error;
use axum::http::StatusCode;
use sqlx::sqlite::SqlitePool;

#[derive(Clone)]
pub struct Database {
    db_pool: SqlitePool,
}


impl Database {

    pub fn new(db_pool: SqlitePool) -> Self {
        Self {db_pool}
    }

}


impl super::Database for Database {

    fn pool(&self) -> &SqlitePool {
        &self.db_pool
    }

    async fn init(&mut self) -> Result<(), Box<dyn Error>> {
        sqlx::migrate!("../rsc/db_migrations/league_members").run(&self.db_pool).await?;
        Ok(())
    }

}
