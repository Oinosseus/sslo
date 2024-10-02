use std::error::Error;
use sqlx::Row;

pub mod league;

pub trait Database {

    /// Ensure database is working correctly
    ///
    /// * initialize a first connection
    /// * check schema and upgrade if necessary
    async fn init(&mut self) -> Result<(), Box<dyn Error>>;


    async fn create_table_if_not_exists(&self, db_pool: &sqlx::SqlitePool) -> Result<(), Box<dyn Error>> {
        let res = sqlx::query("PRAGMA table_list;").fetch_all(db_pool).await?;
        for row in res {
            let mut row_string = String::new();
            println!("Schema={}; name={}; type={}; ncol={}; wr={}; strict={}",
                     row.get(0), row.get(1), row.get(2), row.get(3), row.get(4), row.get(5));
        }
        Ok(())
    }
}
