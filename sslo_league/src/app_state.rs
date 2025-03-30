use std::error::Error;
use std::path::{Path, PathBuf};
use axum_server::tls_rustls::RustlsConfig;
use sslo_lib::error::SsloError;
use crate::db2;
use super::config::Config;


#[derive(Clone)]
pub struct AppState {

    /// embedded config object
    pub config: Config,

    /// path to the sslo database directory
    database_dir: PathBuf,

    /// databases
    pub database: db2::DatabaseManager,
}


impl AppState {

    pub async fn new(config_file_path: &PathBuf) -> Result<Self, SsloError> {

        // config
        let config_toml_path = config_file_path.clone();
        let config = Config::from_file(config_file_path)?;

        // sslo database directory
        let mut database_dir = config_toml_path.clone();
        database_dir.pop();
        database_dir.push(&config.general.database_dir);
        if !database_dir.is_dir() {
            return Err(SsloError::ConfigDatabaseDirInvalid(database_dir.display().to_string()));
        }

        // sqlite databases
        let sqlite_dir = database_dir.join("sqlite_league");
        if !sqlite_dir.exists() {
            if let Err(e) = std::fs::create_dir_all(&sqlite_dir) {
                return Err(SsloError::ConfigCannotCreateSqliteDirectories(e));
            };
        }

        let database = db2::DatabaseManager::new(&sqlite_dir).await?;

        // compile app state
        Ok(AppState {
            database_dir,
            config,
            database,
        })
    }


    /// Returns a RustlsConfig object, or panics
    pub async fn get_rustls_config(&self) -> RustlsConfig {

        // get and check cert file
        let path_cert = self.dbpath(&self.config.http.tls_cert);
        if !path_cert.exists() {
            panic!("Cannot find SSL CERT path: '{}!'", path_cert.display());
        }

        // get and check key file
        let path_key = self.dbpath(&self.config.http.tls_key);
        if !path_cert.exists() {
            panic!("Cannot find SSL KEY path: '{}!'", path_key.display());
        }

        RustlsConfig::from_pem_file(path_cert, path_key).await.unwrap()
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
