use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use thiserror::Error;
use std::str::FromStr;

#[derive(Error, Debug)]
pub enum DatabaseError {

    // #[error("sqlx database pool cannot be retrieved")]
    // PoolUnavailable(),

    #[error("Cannot upgrade weak pointer: {0}")]
    WeakUpgradeProblem(String),

    #[error("no data in table {0} at rowid={1}")]
    RowidNotFound(&'static str, i64),

    #[error("no data in table {0} for {1}")]
    DataNotFound(&'static str, String),

    #[error("low level error from sqlx: {0}")]
    SqlxLowLevelError(#[from] sqlx::Error),

    #[error("low level error at database migration: {0}")]
    SqlxMigrationError(#[from] sqlx::migrate::MigrateError),

    #[error("data-lock failed: {0}")]
    DataLockIssue(String),
}

impl DatabaseError {
    pub fn is_not_found_type(&self) -> bool {
        match self {
            DatabaseError::RowidNotFound(_, _) => true,
            DatabaseError::DataNotFound(_, _) => true,
            _ => false,
        }
    }
}

/// When db_path is None, the pool is generated in memory
pub fn get_pool(db_path: Option<&str>) -> SqlitePool {

    match db_path {
        None => {
            let sqlite_opts = SqliteConnectOptions::from_str(":memory:").unwrap();
            let pool = SqlitePoolOptions::new()
                .min_connections(1)
                .max_connections(1)  // default is 10
                .idle_timeout(None)
                .max_lifetime(None)
                .connect_lazy_with(sqlite_opts);
            pool
        },
        Some(db_path) => {
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
        },
    }
}