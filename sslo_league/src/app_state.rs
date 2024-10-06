use std::error::Error;
use std::path::PathBuf;
use db::league::League;
use crate::db;
use crate::db::Database;
use super::config::Config;

/// Prepending a path to a relative path
/// When rel_path is a relative path then rel_path_prepend is prepended,
/// otherwhise (when rel_path is absolute) rel_path is returned.
pub fn abspath<T>(rel_path_prepend: &PathBuf, rel_path: &T) -> PathBuf {
    let mut abs_path = PathBuf::new();
    abs_path.push(rel_path_prepend);
    abs_path.push(rel_path);  // if path is absolut, it overwrites the original (according to documentation)
    abs_path
}

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
        let pool_league = db::create_db_pool(abspath(&config.database.dir, "league.db"));
        let db_league = League::new(pool_league);

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
        abspath(&self.config_toml_path, path)
    }
}