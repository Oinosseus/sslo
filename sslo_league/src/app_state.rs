use serde::de::Error;
use db::league::League;
use crate::db;
use super::config::Config;

#[derive(Clone)]
pub struct AppState {
    db_league: League,
}

impl AppState {

    pub fn new(config: &Config) -> Result<Self, impl Error> {
        let db_league = League::new(&config.database.sql_dir)?;

        Ok(AppState {
            db_league,
        })
    }
}