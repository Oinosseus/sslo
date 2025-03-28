use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SsloError {

    #[error("{0}")]
    GeneralError(String),

    /// config file path, io error
    #[error("failed to read config file '{0}': {1}")]
    ConfigFileUnreadable(String, io::Error),

    /// config file path, toml error
    #[error("failed to parse config file '{0}': {1}")]
    ConfigFileUnparsable(String, toml::de::Error),

    /// database_dir path
    #[error("Config database_dir is not a valid directory path: '{0}'")]
    ConfigDatabaseDirInvalid(String),

    #[error("failed to create sqlite directories: {0}")]
    ConfigCannotCreateSqliteDirectories(#[from] io::Error),


    #[error("Cannot upgrade weak pointer: {0}")]
    WeakUpgradeProblem(String),

    /// tablename, columnname, id
    #[error("no data in table {0} for {1}={2}")]
    DatabaseIdNotFound(&'static str, &'static str, i64),

    /// tablename, columnname, data
    #[error("no data in table {0} for {1}={2}")]
    DatabaseDataNotFound(&'static str, &'static str, String),

    #[error("error from sqlx: {0}")]
    DatabaseSqlx(#[from] sqlx::Error),

    #[error("error at database migration: {0}")]
    DatabaseMigrationError(#[from] sqlx::migrate::MigrateError),
}

impl SsloError {
    pub fn is_db_not_found_type(&self) -> bool {
        match self {
            SsloError::DatabaseIdNotFound(_, _, _) => true,
            SsloError::DatabaseDataNotFound(_, _, _) => true,
            _ => false,
        }
    }
}

