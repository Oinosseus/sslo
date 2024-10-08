use std::error::Error;
use sqlx::sqlite::SqlitePool;


#[derive(Clone)]
pub struct League {
    db_pool: SqlitePool,
}


impl League {

    pub fn new(db_pool: SqlitePool) -> Self {
        Self {db_pool}
    }

}


impl super::Database for League {

    async fn init(&mut self) -> Result<(), Box<dyn Error>> {
        self.create_table_if_not_exists(&self.db_pool).await?;

        Ok(())
    }

}