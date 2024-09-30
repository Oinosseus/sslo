use std::error::Error;
use std::path::PathBuf;

#[derive(Clone)]
pub struct League {
    db_pool: sqlx::SqlitePool,
}

impl League {

    pub fn new(db_path: &PathBuf) -> Result<Self, impl Error> {

        let db_path : &str = db_path.to_str().ok_or(Err("Cannot parse db path!"))?;

        let db_pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(5)
            .acquire_time_level(log::LevelFilter::Debug)
            .acquire_slow_level(log::LevelFilter::Warn)
            .max_lifetime(Some(std::time::Duration::from_secs(600)))
            .connect(db_path)?;

        Ok(League {db_pool})
    }
}