use std::error::Error;
use sqlx::{Row, SqlitePool};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

pub mod league;

pub fn create_db_pool(db_path: &str) -> SqlitePool {

    let db_conn_opts = SqliteConnectOptions::new()
        .filename(db_path)
        .locking_mode(sqlx::sqlite::SqliteLockingMode::Exclusive)
        .create_if_missing(true)
        .optimize_on_close(true, 400)
        .analysis_limit(Some(400));

    let db_pool_options = SqlitePoolOptions::new()
        .max_connections(5)
        .acquire_time_level(log::LevelFilter::Debug)
        .acquire_slow_level(log::LevelFilter::Warn)
        .max_lifetime(Some(std::time::Duration::from_secs(600)));
    // .connect_lazy(&db_path.to_string_lossy())?;

    SqlitePool::connect_lazy_with(db_conn_opts)
}


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
            let row_schema: String = row.get(0);
            let row_name: String = row.get(1);
            let row_type: String = row.get(2);
            let row_ncol: i64 = row.get(3);
            let row_wr: i64 = row.get(4);
            let row_strict: bool = row.get(5);
            println!("Schema={}; name={}; type={}; ncol={}; wr={}; strict={}",
                     row_schema, row_name, row_type, row_ncol, row_wr, row_strict);
        }
        Ok(())
    }
}
