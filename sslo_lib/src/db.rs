use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {

    #[error("sqlx database pool cannot be retrieved")]
    PoolUnavailable(),

    #[error("no data in table {0} at rowid={1}")]
    RowidNotFound(&'static str, i64),

    #[error("low level error from sqlx: {0}")]
    SqlxLowLevelError(#[from] sqlx::Error),

    #[error("data-lock failed: {0}")]
    DataLockIssue(String),
}

impl DatabaseError {
    pub fn is_rowid_not_found(&self) -> bool {
        match self {
            DatabaseError::RowidNotFound(_, _) => true,
            _ => false,
        }
    }
}