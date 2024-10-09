use std::error::Error;
use std::path::{Path, PathBuf};
use crate::db;
use crate::db::Database;
use super::config::Config;


fn ensure_dir_exists(dir_path: &Path) -> Result<(), Box<dyn Error>> {
    if !dir_path.exists() {
        std::fs::create_dir_all(dir_path)?;
    }
    Ok(())
}


#[derive(Clone)]
pub struct AppState {

    /// embedded config object
    pub config: Config,

    /// path to the sslo database directory
    database_dir: PathBuf,

    /// databases
    db_users: db::users::Database,
}


impl AppState {

    pub fn new(config_file_path: &PathBuf) -> Result<Self, Box<dyn Error>> {

        // config
        let config_toml_path = config_file_path.clone();
        let config = Config::from_file(config_file_path)?;

        // sslo database directory
        let mut database_dir = config_toml_path.clone();
        database_dir.pop();
        database_dir.push(&config.general.database_dir);
        if !database_dir.is_dir() {  // check if db exists
            let msg = format!("Config database_dir is not a valid directory path: '{}!", database_dir.display());
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, msg).into());
        }

        // sqlite databases
        let sqlite_dir = database_dir.join("sqlite");
        ensure_dir_exists(sqlite_dir.as_path())?;
        let pool_users = db::create_db_pool(sqlite_dir.join("users.db").to_str().unwrap());
        let db_users = db::users::Database::new(pool_users);

        // compile app state
        Ok(AppState {
            database_dir,
            config,
            db_users,
        })
    }


    pub async fn init(&mut self) -> Result<(), Box<dyn Error>> {
        self.db_users.init().await?;
        Ok(())
    }


    /// Relate a path to the sslo database directory and return.
    /// When the given path already absolute, it is returned unchanged.
    pub fn dbpath<P: AsRef<Path>>(&self, path: &P) -> PathBuf {
        let mut p = PathBuf::new();
        p.push(&self.database_dir);
        p.push(path);
        return p;
    }
}
