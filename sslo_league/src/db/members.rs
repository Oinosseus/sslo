use std::error::Error;
use axum::http::StatusCode;
use sqlx::sqlite::SqlitePool;
use sqlx::Row;

#[derive(Clone)]
pub struct Database {
    db_pool: SqlitePool,
}


impl Database {

    pub fn new(db_pool: SqlitePool) -> Self {
        Self {db_pool}
    }


    /// Create a new token for email login
    /// This creates a new entry in the email table or, if already existing,
    /// creates a new token (when the old token is already invalid).
    /// Returns the new token (to be sent by email)
    pub async fn new_email_login_token(&self, email: &str) -> Result<String, Box<dyn Error>> {

        // create new token
        let token = sslo_lib::token::Token::new()?;

        // check if email exists
        let res = sqlx::query("SELECT rowid, token_creation, token_last_usage FROM emails WHERE email=$1;")
            .bind(email)
            .fetch_all(&self.db_pool)
            .await?;

        // create new entry
        if res.len() == 0 {
            todo!();
        // updateb entry
        } else if res.len() == 1 {
            let id :i64 = res[0].get("rowid");
            let token_creation = chrono::NaiveDateTime::parse_from_str(res[0].get("token_creation"))?;
            todo!();
        } else {
            log::error!("Unexpected multiple database entries for members.email.email='{}'!", email);
            Err(String::from("requested email is not unique!"))?
        }
    }

}


impl super::Database for Database {

    fn pool(&self) -> &SqlitePool {
        &self.db_pool
    }

    async fn init(&mut self) -> Result<(), Box<dyn Error>> {
        sqlx::migrate!("../rsc/db_migrations/league_members").run(&self.db_pool).await?;
        Ok(())
    }

}
