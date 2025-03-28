use std::path::Path;
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::str::FromStr;

/// When db_path is None, the pool is generated in memory
pub fn get_pool(db_path: Option<&Path>) -> SqlitePool {

    match db_path {
        None => {
            let sqlite_opts = SqliteConnectOptions::from_str(":memory:")
                .unwrap()
                .foreign_keys(true);
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
                .analysis_limit(Some(400))
                .foreign_keys(true);
            pool_opts.connect_lazy_with(conn_opts)
        },
    }
}
