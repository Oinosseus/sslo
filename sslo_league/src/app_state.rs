use std::error::Error;
use std::path::PathBuf;
use db::league::League;
use crate::db;
use crate::db::Database;
use super::config::Config;

#[derive(Clone)]
pub struct AppState {
    config_toml_path: PathBuf,
    pub config: Config,
    db_league: League,
}

impl AppState {

    pub fn new(config_file_path: &PathBuf) -> Result<Self, Box<dyn Error>> {

        // config
        let config_toml_path = config_file_path.clone();
        let config = Config::from_file(config_file_path)?;

        // databases
        let db_league = League::new(db::create_db_pool(self, "league.db"))?;

        // compile app state
        Ok(AppState {
            config_toml_path,
            config,
            db_league,
        })
    }


    pub async fn init(&mut self) -> Result<(), Box<dyn Error>> {
        self.db_league.init().await?;
        Ok(())
    }


    /// Assume paths relative from the config.toml file location and return as absolute path
    /// When the given path is already absolute, nothing is changed.
    pub fn abspath<T>(&self, path: &T) -> PathBuf {
        let mut abs_path = PathBuf::new();
        abs_path.push(&self.config_toml_path);
        abs_path.push(path);  // if path is absolut, it overwrites the original (according to documentation)
        abs_path
    }
}