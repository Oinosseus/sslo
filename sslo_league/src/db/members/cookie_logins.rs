use chrono::{DateTime, Utc};
use sqlx::Error::RowNotFound;
use sqlx::SqlitePool;
use sslo_lib::token;

/// A struct that represents a whole table row
#[derive(sqlx::FromRow)]
struct DbRow {
    pub rowid: i64,
    pub user: i64,
    pub token: String,
    pub creation: DateTime<Utc>,
    pub last_useragent: Option<String>,
    pub last_usage: Option<DateTime<Utc>>,
}


pub struct CookieLogin {
    pool: SqlitePool,
    row: DbRow,
}

impl CookieLogin {


    /// Returns a string that shall be used as SET-COOKIE http header value
    /// The Id of the new CookieLogin object and the token is combined within the cookie value
    pub async fn new(pool: &SqlitePool, user: &super::users::User) -> Option<String> {

        // generate token
        let token = match token::Token::generate(token::TokenType::Quick) {
            Ok(t) => t,
            Err(e) => {
                log::error!("Failed to generate token: {}", e);
                return None;
            }
        };
        let token_creation = chrono::DateTime::<chrono::Utc>::from(Utc::now());

        // save to DB
        let res: DbRow = match sqlx::query_as("INSERT INTO cookie_logins (user, token, creation) VALUES ($1, $2, $3) RETURNING rowid,*;")
            .bind(user.rowid())
            .bind(token.encrypted)
            .bind(&token_creation)
            .fetch_one(pool)
            .await {
            Ok(row) => row,
            Err(e) => {
                log::error!("Failed to insert into db.members.cookie_logins: {}", e);
                return None;
            }
        };

        // create cookie
        log::info!("Creating new login cookie for db.members.users.rowid={} ({})", user.rowid(), user.name_ref());
        let cookie = format!("cookie_login={}:{}; HttpOnly; Max-Age=31536000; SameSite=Strict; Partitioned; Secure; Path=/;",
                             res.rowid, token.decrypted);
        Some(cookie)
    }


    /// Find an CookieLogin in the database that was most recently used for login
    pub async fn from_last_usage(pool: SqlitePool, user: &super::users::User) -> Option<Self> {
        let row: DbRow = match sqlx::query_as("SELECT rowid,* FROM cookie_logins WHERE user=$1 ORDER BY last_usage DESC LIMIT 1;")
            .bind(user.rowid())
            .fetch_one(&pool)
            .await {
            Ok(x) => x,
            Err(RowNotFound) => return None,
            Err(e) => {
                log::error!("Failed sql query: {}", e);
                return None;
            },
        };

        Some(Self{ pool, row })
    }


    /// Get an item from the database
    pub async fn from_id(pool: SqlitePool, rowid: i64) -> Option<Self> {

        let mut rows : Vec<DbRow> = match sqlx::query_as("SELECT rowid, * FROM cookie_logins WHERE rowid=$1 LIMIT 2;")
            .bind(rowid)
            .fetch_all(&pool)
            .await {
            Ok(x) => x,
            Err(e) => {
                log::error!("Failed to get db.members.cookie_logins.rowid={} from database: {}", rowid, e);
                return None;
            },
        };

        // fail on multiple results
        if rows.len() > 1 {
            log::error!("Multiple database entries for db.members.cookie_logins.rowid={}", rowid);
            return None;
        }

        // return data
        if let Some(row) = rows.pop() {
            Some(Self{pool, row})
        } else {
            log::debug!("db.members.cookie_login.rowid={} not found", rowid);
            None
        }
    }


    /// Get an item from the database
    /// This verifies the token and automatically updates usage info
    pub async fn from_cookie(pool: SqlitePool, useragent: String, cookie: &str) -> Option<Self> {
        let process_duration = std::time::Instant::now();

        // quick check
        if cookie.find("cookie_login=").is_none() {
            return None;
        };

        // chop cookie string
        let re = regex::Regex::new(r"^cookie_login=([0-9]+):(.*)$").unwrap();
        let groups = match re.captures(cookie) {
            Some(x) => x,
            None => {
                log::warn!("Invalid cookie format: '{}'", cookie);
                return None;
            }
        };
        let id: i64 = match groups.get(1)?.as_str().parse::<i64>() {
            Ok(id) => id,
            Err(_) => return None,
        };
        let token_decrypted: String = groups.get(2)?.as_str().into();

        // find id in database
        let mut item = Self::from_id(pool, id).await?;

        // verify token
        let token = sslo_lib::token::Token::new(token_decrypted, item.row.token.clone());
        if !token.verify() {
            return None;
        }

        // update usage
        item.report_usage(useragent).await;

        Some(item)
    }


    /// Update usage info
    pub async fn report_usage(&mut self, useragent: String) {

        // update db
        let row = match sqlx::query_as("UPDATE cookie_logins SET last_useragent=$1, last_usage=$2 WHERE rowid=$3 RETURNING rowid,*;")
            .bind(useragent)
            .bind(Utc::now())
            .bind(self.row.rowid)
            .fetch_one(&self.pool)
            .await {
            Ok(r) => r,
            Err(e) => {
                log::error!("Could not update db.members.cookie_login.rowid={}: {}", self.row.rowid, e);
                return;
            }
        };

        // store updated values
        self.row = row;
    }


    /// Delete cookie from database and returns a value for a SET-COOKIE http header value that shall be transmitted
    pub async fn delete(self) -> String {

        match sqlx::query("DELETE FROM cookie_logins WHERE rowid = $1;")
            .bind(self.row.rowid)
            .execute(&self.pool)
            .await {
            Ok(_) => {},
            Err(e) => {
                log::error!("Failed to delete db.members.cookie_login.rowid={}: {}", self.row.rowid, e);
            }
        }

        "cookie_login=\"\"; HttpOnly; Max-Age=-1; SameSite=Strict; Partitioned; Secure; Path=/;".to_string()
    }


    pub fn last_useragent(&self) -> Option<&String> { self.row.last_useragent.as_ref() }

    pub fn last_usage(&self) -> Option<DateTime<Utc>> { self.row.last_usage }

    /// get a users::User from the database
    pub async fn user(&self) -> Option<super::users::User> {
        super::users::User::from_id(self.pool.clone(), self.row.rowid).await
    }
}
