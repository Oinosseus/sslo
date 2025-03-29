use std::path::PathBuf;
use serde::Deserialize;
use sslo_lib::error::SsloError;

#[derive(Deserialize, Clone)]
pub struct Config {

    /// General configuration options
    pub general: General,

    /// Configuration for the http(s) server(s)
    pub http: Http,
}


impl Config {

    /// Read config from a toml file
    pub fn from_file(file_path: &PathBuf) -> Result<Self, SsloError> {
        let toml_content = std::fs::read_to_string(&file_path).or_else(|e| {
            let e = SsloError::ConfigFileUnreadable(file_path.display().to_string(), e);
            Err(e)
        })?;
        let config : Self = toml::from_str(&toml_content).or_else(|e| {
            let e = SsloError::ConfigFileUnparsable(file_path.display().to_string(), e);
            Err(e)
        })?;
        Ok(config)
    }
}


#[derive(Deserialize, Clone)]
pub struct General {

    /// The directory where the SSLO database is located
    /// This can be an absolute path or relative path.
    /// A relative path is assumed to be relative to the config.toml file
    /// All other relative paths in this file are considered to be relative in relation to this.
    pub database_dir: PathBuf,
}


#[derive(Deserialize, Clone)]
pub struct Http {

    /// The port to run the http server onto
    pub port_http: u16,

    /// The port to run the https server onto
    pub port_https: u16,

    /// Path to the TLS cert file in PEM format
    pub tls_cert: PathBuf,

    /// Path to the TLS key file in PEM format
    pub tls_key: PathBuf,
}
