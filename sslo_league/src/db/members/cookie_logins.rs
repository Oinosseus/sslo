use std::error::Error;
use chrono::Utc;
use rand::RngCore;
use sqlx::SqlitePool;
use sslo_lib::token::Token;

/// A struct that represents a whole table row
#[derive(sqlx::FromRow)]
pub struct Item {
    pub rowid: i64,
    pub user: i64,
    pub token: String,
    pub creation: String,
    pub last_user_agent: Option<String>,
    pub last_usage: Option<String>,
}


#[derive(Clone)]
pub struct Table {
    db_pool: SqlitePool,
}


impl Table {
    pub fn new(db_pool: SqlitePool) -> Self {
        Self { db_pool }
    }

    /// Returns a string that shall be used as SET-COOKIE http header value
    pub async fn new_cookie(&self, user: i64) -> Result<String, Box<dyn Error>> {

        // generate token
        let token: Token = Token::generate()?;
        let token_creation = chrono::DateTime::<chrono::Utc>::from(Utc::now());

        // save to DB
        let res: Item = sqlx::query_as("INSERT INTO cookie_logins (user, token, creation) VALUES ($1, $2, $3) RETURNING rowid,*;")
            .bind(user)
            .bind(token.encrypted)
            .bind(crate::db::time2string(&token_creation))
            .fetch_one(&self.db_pool)
            .await?;

        // create cookie
        let cookie = format!("cookie_login={}:{}; HttpOnly; Max-Age=31536000; SameSite=Strict; Partitioned; Secure; Path=/;",
                             res.rowid, token.decrypted);

        Ok(cookie)
    }


    pub async fn from_id(&self, id: i64) -> Option<Item> {

        let mut res : Vec<Item> = match sqlx::query_as("SELECT rowid, * FROM cookie_logins WHERE rowid=$1 LIMIT 2;")
            .bind(id)
            .fetch_all(&self.db_pool)
            .await {
            Ok(x) => x,
            Err(e) => {
                log::error!("Failed to request database: {}", e);
                return None;
            },
        };

        // fail on multiple results
        if res.len() > 1 {
            log::error!("Multiple database entries={}", id);
            return None;
        }

        res.pop()
    }


    pub async fn from_cookie(&self, cookie: &str) -> Option<Item> {

        // quick chek
        if cookie.find("cookie_login=").is_none() {
            return None;
        };

        // chop cookie string
        let re = regex::Regex::new(r"^cookie_login=([0-9]+):(.*)$").unwrap();
        let groups = re.captures(cookie)?;
        let id: i64 = match groups.get(1)?.as_str().parse::<i64>() {
            Ok(id) => id,
            Err(_) => return None,
        };
        let token_decrypted: String = groups.get(2)?.as_str().into();

        // find id in database
        let item = self.from_id(id).await?;

        // verify token
        let token = sslo_lib::token::Token::new(token_decrypted, item.token.clone());
        if !token.verify() {
            return None;
        }

        Some(item)
    }
}
