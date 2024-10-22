mod email;

use std::error::Error;
use axum::http::StatusCode;
use sqlx::sqlite::SqlitePool;
use sqlx::Row;
use crate::db::Database;

#[derive(Clone)]
pub struct DbMembers {
    db_pool: SqlitePool,
    pub tbl_email: email::TblEmail,
}


impl DbMembers {

    pub fn new(db_pool: SqlitePool) -> Self {
        Self {
            db_pool: db_pool.clone(),
            tbl_email: email::TblEmail::new(db_pool.clone()),
        }
    }

}


impl super::Database for DbMembers {

    fn pool(&self) -> &SqlitePool {
        &self.db_pool
    }

    async fn init(&mut self) -> Result<(), Box<dyn Error>> {
        sqlx::migrate!("../rsc/db_migrations/league_members").run(&self.db_pool).await?;
        Ok(())
    }

}
