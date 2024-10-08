use std::error::Error;
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
        self.create_table_if_not_exists(&self.db_pool).await?;

        Ok(())
    }

}
