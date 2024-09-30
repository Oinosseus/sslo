use std::error::Error;
use std::path::PathBuf;
use serde::Deserialize;
use thiserror::Error;

#[derive(Deserialize)]
pub struct Config {

    /// Configuration for the http(s) server(s)
    pub http: Http,

    /// Configuration for the SQL databases
    pub database: Database,
}

impl Config {

    /// Read config from a toml file
    pub fn from_file(file_path: PathBuf) -> Result<Self, impl Error> {
        let toml_content = std::fs::read_to_string(file_path)?;
        toml::from_str(&toml_content)
    }
}

#[derive(Deserialize)]
pub struct Http {

    /// The port to run the http server onto
    pub port_http: u16,

    /// the port to run the https server onto
    pub port_https: u16,

    /// Path to the SSL cert file in PEM format
    pub ssl_cert: PathBuf,

    /// Path to the SSL key file in PEM format
    pub ssl_key: PathBuf,
}

#[derive(Deserialize)]
pub struct Database {

    /// The directory where all SQL databases are stored
    pub sql_dir: PathBuf,
}