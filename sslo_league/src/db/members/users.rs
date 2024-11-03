use std::error::Error;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use sslo_lib::token;
use crate::user_grade;

#[derive(sqlx::FromRow)]
struct DbRow {
    pub rowid: i64,
    pub name: String,
    pub promotion_authority: user_grade::PromotionAuthority,
    pub promotion: user_grade::Promotion,
    pub last_lap: Option<DateTime<Utc>>,
    pub email: Option<String>,
    pub email_token: Option<String>,
    pub email_token_creation: Option<DateTime<Utc>>,
    pub email_token_consumption: Option<DateTime<Utc>>,
    pub password: Option<String>,
    pub password_last_usage: Option<DateTime<Utc>>,
    pub password_last_user_agent: Option<String>,
}


pub struct User {
    db_row: DbRow,
    db_pool: SqlitePool,
}

impl User {

    /// Retrieve a User object from database
    pub async fn from_id(db_pool: SqlitePool, rowid: i64) -> Option<Self> {
        let mut rows = match sqlx::query_as("SELECT rowid,* FROM users WHERE rowid = $1")
            .bind(rowid)
            .fetch_all(&db_pool)
            .await {
            Ok(r) => r,
            Err(e) => {
                log::error!("Failed to query database: {}", e);
                return None;
            }
        };
        if let Some(db_row) = rows.pop() {
            Some(Self {db_row, db_pool})
        } else {
            None
        }
    }


    /// Retrieve a User object from database
    /// Returns None when email is ambiguous
    pub async fn from_email(db_pool: SqlitePool, email: &str) -> Option<Self> {
        let mut rows = match sqlx::query_as("SELECT rowid,* FROM users WHERE email = $1 LIMIT 2;")
            .bind(email)
            .fetch_all(&db_pool)
            .await {
            Ok(r) => r,
            Err(e) => {
                log::error!("Failed to query database: {}", e);
                return None;
            }
        };
        if rows.len() > 1 {
            log::error!("Ambiguous db.members.users.email='{}'", email);
            return None;
        }
        if let Some(db_row) = rows.pop() {
            Some(Self {db_row, db_pool})
        } else {
            None
        }
    }


    /// datetime of last driven lap
    pub fn last_lap(&self) -> Option<DateTime<Utc>> { self.db_row.last_lap }

    /// name of the user
    pub fn name_ref(&self) -> &str { &self.db_row.name }

    pub fn promotion(&self) -> user_grade::Promotion { self.db_row.promotion.clone() }

    pub fn promotion_authority(&self) -> user_grade::PromotionAuthority {
        self.db_row.promotion_authority.clone()
    }

    /// database rowid
    pub fn rowid(&self) -> i64 { self.db_row.rowid }


    /// Insert new entry into users table
    pub async fn new_from_email(db_pool: SqlitePool, email: &str) -> Option<Self> {
        match sqlx::query_as("INSERT INTO users (name, email) VALUES ($1, $2) RETURNING rowid, *;")
            .bind(email)
            .bind(email)
            .fetch_one(&db_pool)
            .await {
                Ok(db_row) => {
                    log::info!("Creating user from email '{}'", email);
                    Some(Self { db_row, db_pool })
                },
                Err(e) => {
                    log::error!("Unable to create new row into db.members.users: {}", e);
                    None
                }
        }
    }


    /// Consume the email token and return true on success
    pub async fn redeem_email_token(&mut self, plain_token: String) -> bool {

        // get some basics
        let time_now = &crate::helpers::now();
        let time_token_outdated = time_now.clone()
            .checked_add_signed(chrono::TimeDelta::hours(-1))
            .unwrap();  // subtracting one hour cannot fail, technically

        // check if token is existing
        let crypted_token: String = match &self.db_row.email_token {
            Some(token_ref) => token_ref.clone(),
            None => {
                log::warn!("No token set for db.members.users.rowid={}", self.db_row.rowid);
                return false;
            },
        };

        // check if token was already used
        if let Some(email_token_consumption) = self.db_row.email_token_consumption {
            log::warn!("Deny redeeming token for db.members.users.rowid={}, because already consumed at {}",
                self.db_row.rowid,
                email_token_consumption,
            );
            return false;
        }

        // check if token is still valid
        match self.db_row.email_token_creation {
            None => {
                log::error!("Invalid token creation time for db.members.users.rowid={}", self.db_row.rowid);
                return false;
            },
            Some(token_creation) => {
                if token_creation < time_token_outdated {  // token outdated
                    log::warn!("Deny redeeming token for db.members.users rowid={}, because token outdated", self.db_row.rowid);
                    return false;
                }
            },
        };

        // verify token
        let token = token::Token::new(plain_token, crypted_token);
        if !token.verify() {
            log::warn!("Deny redeeming token for db.members.users.rowid={}, because token invalid!", self.db_row.rowid);
            return false;
        }

        // redeem token
        match sqlx::query("UPDATE users SET email_token=NULL, email_token_consumption=$1 WHERE rowid=$2;")
            .bind(&time_now)
            .bind(self.db_row.rowid)
            .execute(&self.db_pool)
            .await {
            Ok(_) => {},
            Err(e) => {
                log::warn!("Database error at redeeming token for db.members.users.rowid={}, because: {}", self.db_row.rowid, e);
                return false;
            },
        };

        // redeeming seem to be fine
        log::info!("User email token redeemed: rowid={}, name={}", self.db_row.rowid, self.name_ref());
        true
    }


    /// Create a new token for email login
    /// Creates a new token (if the old token is not still valid).
    /// Returns the new token (to be sent by email)
    pub async fn update_email_login_token(&mut self) -> Option<String> {

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
        if let Some(token_creation) = self.db_row.email_token_creation {
            if token_creation > time_token_outdated {               // token is still valid
                if self.db_row.email_token_consumption.is_none() {    // token is not used, yet
                    log::warn!("Not generating new email login token for user {}:'{}' because last token is still active.", self.db_row.rowid, self.name_ref());
                    return None;
                }
            }
        }

        // update
        match sqlx::query("UPDATE users SET email_token=$1, email_token_creation=$2, email_token_consumption=NULL WHERE rowid=$3;")
            .bind(&token.encrypted)
            .bind(&time_now)
            .bind(self.db_row.rowid)
            .execute(&self.db_pool)
            .await {
            Ok(_) => {},
            Err(e) => {
                log::error!("Failed to update database members.emails: {}", e);
                return None;
            },
        }

        // save changes and return plain token
        self.db_row.email_token = Some(token.encrypted);
        self.db_row.email_token_creation = Some(time_now);
        self.db_row.email_token_consumption = None;
        Some(token.decrypted)
    }


    /// set new name
    pub async fn update_name(&mut self, name: String) -> Result<(), Box<dyn Error>> {
        match sqlx::query("UPDATE users SET name = $1 WHERE rowid = $2;")
            .bind(&name)
            .bind(self.db_row.rowid)
            .execute(&self.db_pool)
            .await {
            Ok(_) => {
                self.db_row.name = name;
                Ok(())
            },
            Err(e) => {
                log::error!("Failed to update db.members.users.rowid={}", self.db_row.rowid);
                Err(e)?
            }
        }
    }
}
