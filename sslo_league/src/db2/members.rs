mod users;
mod cookie_logins;

use std::sync::Arc;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;


/// The members database
pub struct Members {
    pool: SqlitePool,
}


impl Members {

    /// Connecting to the database file
    pub fn new(db_path: &str) -> Arc<Self> {

        // set up a db pool
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
        let pool = pool_opts.connect_lazy_with(conn_opts);

        // setup database object
        Arc::new(Self {
            pool
        })
    }

    /// returning a pool object (only used for submodules)
    fn pool(&self) -> SqlitePool { self.pool.clone() }
}