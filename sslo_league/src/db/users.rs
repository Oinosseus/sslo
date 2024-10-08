use std::borrow::Cow;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use sqlx::error::BoxDynError;
use sqlx::migrate::{Migration, MigrationSource, MigrationType};
use sqlx::sqlite::SqlitePool;

#[derive(Clone)]
pub struct Users {
    db_pool: SqlitePool,
}


impl Users {

    pub fn new(db_pool: SqlitePool) -> Self {
        Self {db_pool}
    }

}


impl super::Database for Users {

    async fn init(&mut self) -> Result<(), Box<dyn Error>> {
        sqlx::migrate!("../rsc/db_migrations/league_users").run(&self.db_pool).await?;
        self.create_table_if_not_exists(&self.db_pool).await?;
        Ok(())
    }

}
