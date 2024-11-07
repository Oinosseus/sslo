use std::error::Error;
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

pub mod members;


pub fn create_db_pool(db_path: &str) -> SqlitePool {

    let pool_opts = SqlitePoolOptions::new()
        .max_connections(1)
        .acquire_time_level(log::LevelFilter::Debug)
        .acquire_slow_level(log::LevelFilter::Warn)
        .max_lifetime(Some(std::time::Duration::from_secs(600)));

    let conn_opts = SqliteConnectOptions::new()
        .filename(db_path)
        .locking_mode(sqlx::sqlite::SqliteLockingMode::Exclusive)
        .create_if_missing(true)
        .optimize_on_close(true, 400)
        .analysis_limit(Some(400));

    pool_opts.connect_lazy_with(conn_opts)
}


pub trait Database {

    /// Ensure database is working correctly
    ///
    /// * initialize a first connection
    /// * check schema and upgrade if necessary
    async fn init(&mut self) -> Result<(), Box<dyn Error>>;


}
