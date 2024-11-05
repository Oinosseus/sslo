pub mod emails;
pub mod users;
pub mod cookie_logins;
mod steam_users;

use std::error::Error;
use sqlx::sqlite::SqlitePool;

#[derive(Clone)]
pub struct DbMembers {
    db_pool: SqlitePool,
}


impl DbMembers {

    pub fn new(db_pool: SqlitePool) -> Self {
        Self {
            db_pool: db_pool.clone(),
        }
    }


    /// Create a new login cookie
    pub async fn cookie_login_new(&self, user: &users::User) -> Option<String> {
        cookie_logins::CookieLogin::new(&self.db_pool, user).await
    }

    /// Get a CookieLogin from database
    pub async fn cookie_login_from_id(&self, rowid: i64) -> Option<cookie_logins::CookieLogin> {
        cookie_logins::CookieLogin::from_id(self.db_pool.clone(), rowid).await
    }

    /// Get a CookieLogin from database
    pub async fn cookie_login_from_cookie(&self, user_agent: String, cookie: &str) -> Option<cookie_logins::CookieLogin> {
        cookie_logins::CookieLogin::from_cookie(self.db_pool.clone(), user_agent, cookie).await
    }

    /// Gte a CookieLogin from database that was used for most recent login
    pub async fn cookie_login_from_last_usage(&self, user: &users::User) -> Option<cookie_logins::CookieLogin> {
        cookie_logins::CookieLogin::from_last_usage(self.db_pool.clone(), user).await
    }


    /// Get an item from the users database
    pub async fn user_from_id(&self, rowid: i64) -> Option<users::User> {
        users::User::from_id(self.db_pool.clone(), rowid).await
    }


    /// Get an item from the users database
    pub async fn user_from_email(&self, email: &str) -> Option<users::User> {
        users::User::from_email(self.db_pool.clone(), email).await
    }

    /// Get an item from the user database, including token verification
    pub async fn user_from_email_token(&self, email: &str, plain_token: String) -> Option<users::User> {
        users::User::from_email_token(self.db_pool.clone(), email, plain_token).await
    }

    /// Get an item from the user database, including password verification
    pub async fn user_from_email_password(&self, user_agent: String, email: &str, plain_password: String) -> Option<users::User> {
        users::User::from_email_password(self.db_pool.clone(), user_agent, email, plain_password).await
    }

    /// create a new user from email address
    pub async fn user_new_from_email(&self, email: &str) -> Option<users::User> {
        users::User::new_from_email(self.db_pool.clone(), email).await
    }
}


impl super::Database for DbMembers {

    async fn init(&mut self) -> Result<(), Box<dyn Error>> {
        match sqlx::migrate!("../rsc/db_migrations/league_members").run(&self.db_pool).await {
            Ok(_) => {},
            Err(e) => {
                log::error!("Failed to migrate db.members!");
                return Err(e)?;
            }
        };
        Ok(())
    }

}
