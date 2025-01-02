mod users;
mod cookie_logins;

use std::sync::Arc;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use tokio::sync::RwLock;
use crate::db::members::users::User;

/// The members database
pub struct Members {
    pool: SqlitePool,
    tbl_users: Arc<RwLock<users::TableData>>,
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

        // create tables
        let tbl_users = users::TableData::new(&pool);

        // setup database object
        Arc::new(Self {
            pool,
            tbl_users,
        })
    }

    /// returning a pool object (only used for submodules)
    fn pool(&self) -> SqlitePool { self.pool.clone() }

    fn tbl_users(&self) -> users::TableInterface {
        users::TableInterface::new(self.tbl_users.clone())
    }
}
