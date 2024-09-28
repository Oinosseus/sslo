use std::path::PathBuf;
use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigLoadError {
    #[error("Cannot open configuration file: {0}")]
    ReadFileError(#[from] std::io::Error),

    #[error("Cannot parse configuration file: {0}")]
    ParseFileError(#[from] toml::de::Error),
}


#[derive(Deserialize)]
pub struct Config {

    /// Configuration for the http(s) server(s)
    http: Http,
}

impl Config {

    /// Read config from a toml file
    pub fn from_file(file_path: PathBuf) -> Result<Self, ConfigLoadError> {
        let toml_content = std::fs::read_to_string(file_path)?;
        let cfg: Self = toml::from_str(&toml_content)?;
        return Ok(cfg);
    }
}

#[derive(Deserialize)]
struct Http {

    /// The port to run the http server onto
    port_http: u16,

    /// the port to run the https server onto
    port_https: u16,

    /// Path to the SSL cert file in PEM format
    ssl_cert: PathBuf,

    /// Path to the SSL key file in PEM format
    ssl_key: PathBuf,
}