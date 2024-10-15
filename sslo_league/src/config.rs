use std::error::Error;
use std::path::PathBuf;
use serde::Deserialize;


#[derive(Deserialize, Clone)]
pub struct Config {

    /// General configuration options
    pub general: General,

    /// Configuration for the http(s) server(s)
    pub http: Http,

    /// Configuration for sending emails
    pub smtp: Smtp,
}


impl Config {

    /// Read config from a toml file
    pub fn from_file(file_path: &PathBuf) -> Result<Self, Box<dyn Error>> {
        let toml_content = std::fs::read_to_string(file_path)?;
        let config : Self = toml::from_str(&toml_content)?;
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


#[derive(Deserialize, Clone)]
/// Configuration for SMTP email sending server
/// secure STARTTLS SMTP expected -> TLS certificate must be valid
pub struct Smtp {

    /// The email address that shall be used as sender for the SSLO system
    pub email: String,

    /// The hostname of the SMTP server (eg. mail.mydomain.com)
    pub host: String,

    /// The username for login
    pub username: String,

    /// The password for login
    pub password: String,
}
