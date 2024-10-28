use std::error::Error;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use sslo_lib::token;
use crate::db;

/// A struct that represents a whole table row
#[derive(sqlx::FromRow)]
struct Item {
    rowid: i64,
    email: String,
    creation: DateTime<Utc>,
    token: Option<String>,
    token_creation: Option<DateTime<Utc>>,
    token_last_usage: Option<DateTime<Utc>>,
    password: Option<String>,
    password_creation: Option<DateTime<Utc>>,
    password_last_usage: Option<DateTime<Utc>>,
    user: Option<i64>,
}

#[derive(Clone)]
pub struct Table {
    db_pool: SqlitePool,
}

impl Table {

    pub fn new(db_pool: SqlitePool) -> Self {
        Self{ db_pool }
    }


    /// Find a table row by email address
    pub async fn from_email(&self, email: &str) -> Option<Item> {

        let mut res : Vec<Item> = match sqlx::query_as("SELECT rowid, * FROM emails WHERE email=$1 LIMIT 2;")
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
            log::error!("Multiple database entries for members.emails.email={}", email);
            return None;
        }

        res.pop()
    }


    /// Create a new token for email login
    /// This creates a new entry in the email table or, if already existing,
    /// creates a new token (when the old token is already invalid).
    /// Returns the new token (to be sent by email)
    pub async fn new_email_login_token(&self, email: &str) -> Result<String, Box<dyn Error>> {

        // get some basics
        let token = token::Token::generate(token::TokenType::Strong)?;
        let time_now = &crate::helpers::now();
        let time_token_outdated = time_now.clone()
            .checked_add_signed(chrono::TimeDelta::hours(-1))
            .unwrap();  // subtracting one hour cannot fail, technically

        // update entry
        if let Some(existing_row) = self.from_email(email).await {

            // check last token
            if let Some(token_creation) = existing_row.token_creation {
                if token_creation > time_token_outdated {           // token is still valid
                    if existing_row.token_last_usage.is_none() {    // token is not used, yet
                        log::warn!("Not generating new email login token for '{}' because last token is still active.", &email);
                        Err("Last generated token is still active!")?
                    }
                }
            }

            // update
            sqlx::query("UPDATE emails SET token=$1, token_creation=$2, token_last_usage=NULL WHERE rowid=$3;")
                .bind(token.encrypted)
                .bind(&time_now)
                .bind(existing_row.rowid)
                .execute(&self.db_pool)
                .await
                .or_else(|e| {
                    log::error!("Failied to update database members.emails: {}", e);
                    Err(e)
                })?;

            // return
            log::info!("Update token for db.members.emails.email={}", email);
            Ok(token.decrypted)

        // create new
        } else {

            // insert
            sqlx::query("INSERT INTO emails (email, token, token_creation) VALUES ($1, $2, $3)")
                .bind(email)
                .bind(&token.encrypted)
                .bind(&time_now)
                .execute(&self.db_pool)
                .await
                .or_else(|e| {
                   log::error!("Failed to insert into db.members.emails: {}", e);
                    Err(e)
                })?;

            // return
            log::info!("New db.members.emails.email={}", email);
            Ok(token.decrypted)
        }
    }


    /// Try to login with an email address and a plain token (not encrypted)
    /// Returns the item of from the associated user table
    pub async fn from_email_token(&self, email: String, plain_token: String) -> Result<super::users::Item, Box<dyn Error>> {

        // get some basics
        let time_now = &crate::helpers::now();
        let time_token_outdated = time_now.clone()
            .checked_add_signed(chrono::TimeDelta::hours(-1))
            .unwrap();  // subtracting one hour cannot fail, technically

        // get table row
        let row_email = match self.from_email(&email).await {
            Some(row) => row,
            None => {
                return Err(format!("Email not found '{}'", &email))?
            },
        };

        // check if token is existing
        let crypted_token = match row_email.token {
            Some(token) => token,
            None => {
                log::warn!("No active token for email '{}'", &email);
                return Err(format!("Token not found '{}'", &email))?
            }
        };

        // check if token was already used
        if let Some(last_usage_time) = row_email.token_last_usage {
            log::warn!("Email token for {} was already used at '{}'", &row_email.email, last_usage_time);
            return Err(format!("Email token for {} was already used at '{}'", &row_email.email, last_usage_time))?;
        }

        // check if token is still valid
        let token_creation: DateTime<Utc> = match row_email.token_creation {
            None => {
                log::error!("invalid column 'token_creation' for db.members.emails.rowid={}", row_email.rowid);
                return Err(format!("Invalid column 'token_creation' for email '{}'", &email))?;
            }
            Some(token_creation) => { token_creation },
        };
        if token_creation < time_token_outdated {  // token outdated
            log::warn!("Email token from '{}' outdated since '{}'", token_creation.to_rfc3339(), time_token_outdated.to_rfc3339());
            return Err(format!("Email token from '{}' outdated since '{}'", token_creation.to_rfc3339(), time_token_outdated.to_rfc3339()))?;
        }

        // verify token
        let token = token::Token::new(plain_token, crypted_token);
        if !token.verify() {
            log::warn!("Failed to validate token for email '{}'", email);
            return Err(format!("Failed to validate token for email '{}'", email))?;
        }

        // redeem token
        sqlx::query("UPDATE emails SET token_last_usage=$1 WHERE rowid=$2;")
            .bind(&time_now)
            .bind(row_email.rowid)
            .execute(&self.db_pool)
            .await.or_else(|e| {
                log::error!("Failed to update db.members.emails.rowid={}: {}", row_email.rowid, e);
                return Err(e);
        })?;

        // find according user
        let tbl_usr = super::users::Table::new(self.db_pool.clone());
        let user_item: super::users::Item = match row_email.user {
            Some(user_id) => {
                let row_user = match tbl_usr.from_id(user_id).await {
                    Some(row_user) => row_user,
                    None => {
                        log::error!("Cannot find db.members.user.rowid={} for db.members.emails.rowid={}", user_id, row_email.rowid);
                        return Err(format!("Cannot find db.members.user.rowid={} for db.members.emails.rowid={}", user_id, row_email.rowid))?;
                    }
                };
                row_user
            },
            None => {
                let row_user = tbl_usr.new_item(&row_email.email).await.or_else(|e| {
                    log::error!("Failed to create new user: {}", e);
                    return Err(format!("Failed to create new user: {}", e));
                })?;

                // create user link
                sqlx::query("UPDATE emails SET user=$1 WHERE rowid=$2;")
                    .bind(row_user.rowid)
                    .bind(row_email.rowid)
                    .execute(&self.db_pool)
                    .await.or_else(|e| {
                    log::error!("Failed to update database memebrs.emails.rowid[{}].user={}", row_email.rowid, row_user.rowid);
                    return Err(format!("Failed to update database memebrs.emails.rowid[{}].user={}", row_email.rowid, row_user.rowid));
                })?;

                row_user
            }
        };

        Ok(user_item)
    }

}
