use std::error::Error;
use chrono::{DateTime, Utc};
use rand::RngCore;
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
    pub password_last_useragent: Option<String>,
}


pub struct User {
    row: DbRow,
    pool: SqlitePool,
}

impl User {

    /// Retrieve a User object from database
    pub async fn from_id(pool: SqlitePool, rowid: i64) -> Option<Self> {

        // query
        let mut rows = match sqlx::query_as("SELECT rowid,* FROM sers WHERE rowid = $1 LIMIT 2;")
            .bind(rowid)
            .fetch_all(&pool)
            .await {
            Ok(r) => r,
            Err(e) => {
                log::error!("Failed to query database: {}", e);
                return None;
            }
        };

        // ambiguity check
        if rows.len() > 1 {
            log::error!("Ambiguous rowid for db.members.users.rowid={}", rowid);
            return None;
        }

        // return
        if let Some(row) = rows.pop() { Some(Self {row, pool}) }
        else { None }
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
            Some(Self { row: db_row, pool: db_pool })
        } else {
            None
        }
    }


    /// datetime of last driven lap
    pub fn last_lap(&self) -> Option<DateTime<Utc>> { self.row.last_lap }

    /// name of the user
    pub fn name_ref(&self) -> &str { &self.row.name }

    pub fn promotion(&self) -> user_grade::Promotion { self.row.promotion.clone() }

    pub fn promotion_authority(&self) -> user_grade::PromotionAuthority {
        self.row.promotion_authority.clone()
    }

    pub fn has_password(&self) -> bool { self.row.password.is_some() }

    /// database rowid
    pub fn rowid(&self) -> i64 { self.row.rowid }


    /// Insert new entry into users table
    pub async fn new_from_email(db_pool: SqlitePool, email: &str) -> Option<Self> {
        match sqlx::query_as("INSERT INTO users (name, email) VALUES ($1, $2) RETURNING rowid, *;")
            .bind(email)
            .bind(email)
            .fetch_one(&db_pool)
            .await {
                Ok(db_row) => {
                    log::info!("Creating user from email '{}'", email);
                    Some(Self { row: db_row, pool: db_pool })
                },
                Err(e) => {
                    log::error!("Unable to create new row into db.members.users: {}", e);
                    None
                }
        }
    }


    /// Retrieve a User object from database
    /// This is similar to from_email(), but additionally verifies and consumes the token
    pub async fn from_email_token(db_pool: SqlitePool, email: &str, plain_token: String) -> Option<Self> {

        // get self
        let item: Self = match Self::from_email(db_pool.clone(), email).await {
            Some(x) => x,
            None => {
                return None;
            }
        };

        // get some basics
        let time_now = &crate::helpers::now();
        let time_token_outdated = time_now.clone()
            .checked_add_signed(chrono::TimeDelta::hours(-1))
            .unwrap();  // subtracting one hour cannot fail, technically

        // check if token is existing
        let crypted_token: String = match item.row.email_token {
            Some(token_ref) => token_ref.clone(),
            None => {
                log::warn!("No token set for db.members.users.rowid={} ({})", item.row.rowid, email);
                return None;
            },
        };

        // check if token was already used
        if let Some(email_token_consumption) = item.row.email_token_consumption {
            log::warn!("Deny redeeming token for db.members.users.rowid={} ({}), because already consumed at {}",
                item.row.rowid,
                email,
                email_token_consumption,
            );
            return None;
        }

        // check if token is still valid
        match item.row.email_token_creation {
            None => {
                log::error!("Invalid token creation time for db.members.users.rowid={} ({})", item.row.rowid, email);
                return None;
            },
            Some(token_creation) => {
                if token_creation < time_token_outdated {  // token outdated
                    log::warn!("Deny redeeming token for db.members.users rowid={} ({}), because token outdated", item.row.rowid, email);
                    return None;
                }
            },
        };

        // verify token
        let token = token::Token::new(plain_token, crypted_token);
        if !token.verify() {
            log::warn!("Deny redeeming token for db.members.users.rowid={} ({}), because token invalid!", item.row.rowid, email);
            return None;
        }

        // redeem token
        match sqlx::query("UPDATE users SET email_token=NULL, email_token_consumption=$1 WHERE rowid=$2;")
            .bind(&time_now)
            .bind(item.row.rowid)
            .execute(&db_pool)
            .await {
            Ok(_) => {},
            Err(e) => {
                log::error!("Database error at redeeming token for db.members.users.rowid={} ({}), because: {}", item.row.rowid, email, e);
                return None;
            },
        };

        // redeeming seem to be fine
        log::info!("User email token redeemed: members.users.rowid={} ({})", item.row.rowid, email);
        Self::from_id(db_pool, item.row.rowid).await
    }


    /// Retrieve a User object from database
    /// This is similar to from_email(), but additionally verifies a password
    pub async fn from_email_password(db_pool: SqlitePool, useragent: String, email: &str, plain_password: String) -> Option<Self> {

        // get self
        let item: Self = match Self::from_email(db_pool.clone(), email).await {
            Some(x) => x,
            None => {
                return None;
            }
        };

        // verify password
        if let Some(password) = item.row.password {
            match argon2::verify_encoded(&password, plain_password.as_bytes()) {
                Ok(true) => {},
                Ok(false) => {
                    log::warn!("Deny invalid email password for db.members.users.rowid={} ({})", item.row.rowid, email);
                    return None;
                },
                Err(e) => {
                    log::error!("Failed to verify encoded password for db.members.users.rowid={}", item.row.rowid);
                    return None;
                }
            }
        }

        // update DB
        match sqlx::query("UPDATE users SET password_last_usage=$1, password_last_useragent=$2 WHERE rowid=$3;")
            .bind(Utc::now())
            .bind(useragent)
            .bind(item.row.rowid)
            .execute(&db_pool)
            .await {
                Ok(_) => {},
                Err(e) => {
                    log::error!("Failed to update db.members.users.rowid={}", item.row.rowid);
                    return None;
                }
        }

        // return fresh item
        Self::from_id(db_pool, item.row.rowid).await
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
        if let Some(token_creation) = self.row.email_token_creation {
            if token_creation > time_token_outdated {               // token is still valid
                if self.row.email_token_consumption.is_none() {    // token is not used, yet
                    log::warn!("Not generating new email login token for user {}:'{}' because last token is still active.", self.row.rowid, self.name_ref());
                    return None;
                }
            }
        }

        // update
        match sqlx::query("UPDATE users SET email_token=$1, email_token_creation=$2, email_token_consumption=NULL WHERE rowid=$3;")
            .bind(&token.encrypted)
            .bind(&time_now)
            .bind(self.row.rowid)
            .execute(&self.pool)
            .await {
            Ok(_) => {},
            Err(e) => {
                log::error!("Failed to update database members.emails: {}", e);
                return None;
            },
        }

        // save changes and return plain token
        log::info!("Updating email login token for db.members.users.rowid={} ({})", self.row.rowid, self.name_ref());
        self.row.email_token = Some(token.encrypted);
        self.row.email_token_creation = Some(time_now);
        self.row.email_token_consumption = None;
        Some(token.decrypted)
    }


    /// set new name
    pub async fn update_name(&mut self, name: String) -> Result<(), Box<dyn Error>> {
        match sqlx::query("UPDATE users SET name = $1 WHERE rowid = $2;")
            .bind(&name)
            .bind(self.row.rowid)
            .execute(&self.pool)
            .await {
            Ok(_) => {
                self.row.name = name;
                Ok(())
            },
            Err(e) => {
                log::error!("Failed to update db.members.users.rowid={}", self.row.rowid);
                Err(e)?
            }
        }
    }


    /// set new password
    pub async fn update_password(&mut self, old_password: Option<String>, new_password: String) -> Result<(), ()> {

        // check old password
        if let Some(ref some_password) = self.row.password {
            if let Some(some_old_password) = old_password {
                match argon2::verify_encoded(some_password, &some_old_password.into_bytes()) {
                    Ok(true) => {},
                    Ok(false) => {
                        log::warn!("Deny password change for db.members.users.rowid={} ({}), because old_password does not match!", self.row.rowid, self.name_ref());
                        return Err(());
                    },
                    Err(e) => {
                        log::error!("Argon2 failure at verifying passwords: {}", e);
                        return Err(());
                    }
                }
            } else {
                log::warn!("Deny password change for db.members.users.rowid={} ({}), because no old_password presented!", self.row.rowid, self.name_ref());
                return Err(());
            }
        }

        // check new password strength
        if new_password.len() < 8 {
            log::warn!("Deny setting password for db.members.users.rowid={}, because new password too short!", self.row.rowid);
            return Err(())
        }

        // encrypt new password
        let mut salt: Vec<u8> = vec![0u8; 64];
        rand::thread_rng().fill_bytes(&mut salt);
        let password = match argon2::hash_encoded(&new_password.into_bytes(), &salt, &argon2::Config::default()) {
            Ok(p) => p,
            Err(e) => {
                log::error!("Failed to encrypt password: {}", e);
                return Err(());
            }
        };

        // store to db, return
        match sqlx::query("UPDATE users SET password=$1 WHERE rowid=$2;")
            .bind(password)
            .bind(self.row.rowid)
            .execute(&self.pool)
            .await {
            Err(e) => {
                log::error!("Failed to update db.members.users.rowid={}, because: {}", self.row.rowid, e);
                Err(())
            },
            Ok(_) => {
                log::info!("Changing password of db.members.users.rowid={} ({})", self.row.rowid, self.name_ref());
                Ok(())
            }
        }
    }
}
