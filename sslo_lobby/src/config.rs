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

    /// The url that is used for the http server (e.g. 'localhost:8080')
    pub url_http: String,

    /// The url that is used for the https server (e.g. 'localhost:8443')
    pub url_https: String,

    /// Path to the TLS cert file in PEM format
    pub tls_cert: PathBuf,

    /// Path to the TLS key file in PEM format
    pub tls_key: PathBuf,
}
