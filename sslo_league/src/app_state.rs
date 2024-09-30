use std::error::Error;
use db::league::League;
use crate::db;
use super::config::Config;

#[derive(Clone)]
pub struct AppState {
    db_league: League,
}

impl AppState {

    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let db_league = League::new(&config.database.sql_dir)?;

        Ok(AppState {
            db_league,
        })
    }
}