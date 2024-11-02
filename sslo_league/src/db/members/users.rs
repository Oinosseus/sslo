use std::error::Error;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use sslo_lib::token;

#[derive(sqlx::FromRow)]
pub struct Item {
    pub rowid: i64,
    pub name: String,
    pub promotion_authority: crate::user_grade::PromotionAuthority,
    pub promotion: crate::user_grade::Promotion,
    pub last_lap: Option<DateTime<Utc>>,
    pub email: Option<String>,
    pub email_token: Option<String>,
    pub email_token_creation: Option<DateTime<Utc>>,
    pub email_token_consumption: Option<DateTime<Utc>>,
    pub password: Option<String>,
    pub password_last_usage: Option<DateTime<Utc>>,
    pub password_last_user_agent: Option<String>,
}


#[derive(Clone)]
pub struct Table {
    db_pool: SqlitePool
}

impl Table {
    pub fn new(db_pool: SqlitePool) -> Self { Self {db_pool}}


    /// Insert new entry into users table
    /// Returns rowid on success
    pub async fn new_item(&self, name: &str) -> Result<Item, Box<dyn Error>> {
        let res: Item = sqlx::query_as("INSERT INTO users (name) VALUES ($1) RETURNING rowid, *;")
            .bind(&name)
            .fetch_one(&self.db_pool)
            .await
            .or_else(|e| {
                log::error!("Unable to create new row into db.members.users: {}", e);
                return Err(e);
            })?;
        Ok(res)
    }


    /// Create a new token for email login
    /// Creates a new token (when the old token is not still valid).
    /// Returns the new token (to be sent by email)
    pub async fn new_email_login_token(&self, user_item: &Item) -> Option<String> {

        // get some basics
        let token = match token::Token::generate(token::TokenType::Strong) {
            Ok(t) => t,
            Err(e) => {
                log::error!("Could not generate new token: {}", e);
                return None;
            }
        };
        let time_now = Utc::now();
        let time_token_outdated = time_now.clone()
            .checked_add_signed(chrono::TimeDelta::hours(-1))
            .unwrap();  // subtracting one hour cannot fail, technically

        // check last token
        if let Some(token_creation) = user_item.email_token_creation {
            if token_creation > time_token_outdated {               // token is still valid
                if user_item.email_token_consumption.is_none() {    // token is not used, yet
                    log::warn!("Not generating new email login token for '{}' because last token is still active.", &user_item.rowid);
                    return None;
                }
            }
        }

        // update
        match sqlx::query("UPDATE users SET email_token=$1, email_token_creation=$2, email_token_consumption=NULL WHERE rowid=$3;")
            .bind(token.encrypted)
            .bind(&time_now)
            .bind(user_item.rowid)
            .execute(&self.db_pool)
            .await {
            Ok(_) => {},
            Err(e) => {
                log::error!("Failed to update database members.emails: {}", e);
                return None;
            },
        }

        // return plain token
        Some(token.decrypted)
    }


    /// Find a user by rowid
    pub async fn from_id(&self, rowid: i64) -> Option<Item> {

        let mut res : Vec<Item> = match sqlx::query_as("SELECT rowid, * FROM users WHERE rowid=$1 LIMIT 2;")
            .bind(rowid)
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
            log::error!("Multiple database entries for members.users.rowid={}", rowid);
            return None;
        }

        res.pop()
    }


    /// Find a user by email address
    /// Returns None when ambiguous emails where found
    pub async fn from_email(&self, email: &str) -> Option<Item> {

        // ensure db not mixing up NULL with ""
        if email.len() == 0 { return None; }

        let mut res : Vec<Item> = match sqlx::query_as("SELECT rowid, * FROM users WHERE email=$1 LIMIT 2;")
            .bind(email)
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
            log::error!("Multiple database entries for members.users.email={}", email);
            return None;
        }

        res.pop()
    }


    /// Try to login with an email address and a plain token (not encrypted)
    /// Returns the item of from the associated user table
    pub async fn from_email_token(&self, email: String, plain_token: String) -> Option<Item> {

        // get some basics
        let time_now = &crate::helpers::now();
        let time_token_outdated = time_now.clone()
            .checked_add_signed(chrono::TimeDelta::hours(-1))
            .unwrap();  // subtracting one hour cannot fail, technically

        // get user
        let user_item = self.from_email(&email).await?;

        // check if token is existing
        let crypted_token = match user_item.email_token {
            Some(token) => token,
            None => {
                log::warn!("No token set for email '{}' -> db.members.users.rowid={}", &email, user_item.rowid);
                return None
            },
        };

        // check if token was already used
        if let Some(last_usage_time) = user_item.email_token_consumption {
            log::warn!("Deny reusing email_token for '{}' -> was already used at {}", &email, last_usage_time);
            return None;
        }

        // check if token is still valid
        match user_item.email_token_creation {
            None => {
                log::error!("Invalid token creation time for db.members.users.[rowid={}; email={}]", user_item.rowid, &email);
                return None;
            }
            Some(token_creation) => {
                if token_creation < time_token_outdated {  // token outdated
                    log::warn!("Token from email '{}' outdated: '{}'", &email, token_creation.to_rfc3339());
                    return None;
                }
            },
        };

        // verify token
        let token = token::Token::new(plain_token, crypted_token);
        if !token.verify() {
            log::warn!("Failed to validate token for email '{}'", email);
            return None;
        }

        // redeem token
        match sqlx::query("UPDATE users SET email_token_consumption=$1 WHERE rowid=$2;")
            .bind(&time_now)
            .bind(user_item.rowid)
            .execute(&self.db_pool)
            .await {
            Ok(_) => {},
            Err(e) => {
                log::error!("Failed to update db.members.emails.rowid={}: {}", user_item.rowid, e);
                return None;
            },
        };

        // return updated user item
        self.from_id(user_item.rowid).await
    }


    pub async fn set_name(&self, rowid: i64, name: &str) -> Result<Item, Box<dyn Error>> {
        let item = match sqlx::query_as("UPDATE users SET name = $1 WHERE rowid = $2 RETURNING rowid,*;")
            .bind(name)
            .bind(rowid)
            .fetch_one(&self.db_pool)
            .await {
            Ok(i) => i,
            Err(e) => {
                log::error!("Could not update name for db.members.users.rowid={}", rowid);
                return Err(e)?;
            },
        };

        Ok(item)
    }


    pub async fn set_email(&self, rowid: i64, email: &str) -> Result<Item, Box<dyn Error>> {
        let item = match sqlx::query_as("UPDATE users SET email = $1 WHERE rowid = $2 RETURNING rowid,*;")
            .bind(email)
            .bind(rowid)
            .fetch_one(&self.db_pool)
            .await {
            Ok(i) => i,
            Err(e) => {
                log::error!("Could not update email for db.members.users.rowid={}", rowid);
                return Err(e)?;
            },
        };

        Ok(item)
    }
}