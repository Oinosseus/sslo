use std::error::Error;
use sqlx::SqlitePool;

/// A struct that represents a whole table row
#[derive(sqlx::FromRow)]
struct RowCookieLogin {
    rowid: i64,
    user: i64,
}


#[derive(Clone)]
pub struct TblCookieLogins {
    db_pool: SqlitePool,
}

impl TblCookieLogins {
    pub fn new(db_pool: SqlitePool) -> Self {
        Self { db_pool }
    }

    /// Returns a string that shall be used as SET-COOKIE http header value
    pub async fn new_row(&self, rowid: i64, user: i64) -> Result<String, Box<dyn Error>> {

    }
}
