use std::error::Error;
use sqlx::SqlitePool;
use crate::db;

/// A struct that represents a whole table row
#[derive(sqlx::FromRow)]
struct RowEmail {
    rowid: i64,
    email: String,
    creation: String,
    token: Option<String>,
    token_creation: Option<String>,
    token_last_usage: Option<String>,
    password: Option<String>,
    password_creation: Option<String>,
    password_last_usage: Option<String>,
    user: Option<i64>,
}

#[derive(Clone)]
pub struct TblEmail {
    db_pool: SqlitePool,
}

impl TblEmail {

    pub fn new(db_pool: SqlitePool) -> Self {
        Self{ db_pool }
    }

    /// Find a table row by email address
    pub async fn row_from_email(&self, email: &str) -> Option<RowEmail> {

        let mut res : Vec<RowEmail> = match sqlx::query_as("SELECT rowid, * FROM email WHERE email=$1 LIMIT 2;")
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
            log::error!("Multiple database entries for members.email.email={}", email);
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
        let token = sslo_lib::token::Token::new()?;
        let time_now = &crate::helpers::now();
        let time_token_outdated = time_now.clone()
            .checked_add_signed(chrono::TimeDelta::hours(-1))
            .unwrap();  // subtracting one hour cannot fail, technically

        // update entry
        if let Some(existing_row) = self.row_from_email(email).await {

            // check last token
            if let Some(token_creation_str) = existing_row.token_creation {
                let token_creation = crate::db::string2time(&token_creation_str)?;
                if token_creation > time_token_outdated {           // token is still valid
                    if existing_row.token_last_usage.is_none() {    // token is not used, yet
                        log::warn!("Not generating new email login token for '{}' because last token is still active.", &email);
                        Err("Last generated token is still active!")?
                    }
                }
            }

            // update
            sqlx::query("UPDATE email SET token=$1, token_creation=$2 WHERE rowid=$3;")
                .bind(token.crypted)
                .bind(crate::db::time2string(&time_now))
                .bind(existing_row.rowid)
                .execute(&self.db_pool)
                .await
                .or_else(|e| {
                    log::error!("Failied to update database members.email: {}", e);
                    Err(e)
                })?;

            // return
            log::info!("Update token for db.members.email.email={}", email);
            Ok(token.plain)

        // create new
        } else {

            // insert
            sqlx::query("INSERT INTO email (email, token, token_creation) VALUES ($1, $2, $3)")
                .bind(email)
                .bind(&token.crypted)
                .bind(crate::db::time2string(&time_now))
                .execute(&self.db_pool)
                .await
                .or_else(|e| {
                   log::error!("Failed to insert into db.members.email: {}", e);
                    Err(e)
                })?;

            // return
            log::info!("New db.members.email.email={}", email);
            Ok(token.plain)
        }
    }
}
