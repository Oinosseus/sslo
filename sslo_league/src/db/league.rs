use std::error::Error;
use std::path::PathBuf;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};

#[derive(Clone)]
pub struct League {
    db_pool: SqlitePool,
}

impl League {

    pub fn new(db_pool: SqlitePool) -> Self {
        League {db_pool}
    }

    pub fn new_old(db_path: &PathBuf) -> Result<Self, Box<dyn Error>> {

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

        let db_pool = SqlitePool::connect_lazy_with(db_conn_opts);

        Ok(League {db_pool})
    }
}

impl super::Database for League {

    async fn init(&mut self) -> Result<(), Box<dyn Error>> {
        self.create_table_if_not_exists(&self.db_pool).await?;

        Ok(())
    }

}