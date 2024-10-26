use std::error::Error;
use chrono::Utc;
use rand::RngCore;
use sqlx::SqlitePool;
use sslo_lib::token::Token;

/// A struct that represents a whole table row
#[derive(sqlx::FromRow)]
struct RowCookieLogin {
    rowid: i64,
    user: i64,
    token: String,
    creation: String,
    last_user_agent: Option<String>,
    last_usage: Option<String>,
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
    pub async fn new_cookie(&self, user: i64) -> Result<String, Box<dyn Error>> {

        // generate token
        let token: Token = Token::generate()?;
        let token_creation = chrono::DateTime::<chrono::Utc>::from(Utc::now());

        // save to DB
        let res: RowCookieLogin = sqlx::query_as("INSERT INTO cookie_logins (user, token, creation) VALUES ($1, $2, $3) RETURNING rowid,*;")
            .bind(user)
            .bind(token.encrypted)
            .bind(crate::db::time2string(&token_creation))
            .fetch_one(&self.db_pool)
            .await?;

        // create cookie
        let cookie = format!("login_token={}:{}; HttpOnly; Max-Age=31536000; SameSite=Strict; Partitioned; Secure;",
                             res.rowid, token.decrypted);

        Ok(cookie)
    }
}
