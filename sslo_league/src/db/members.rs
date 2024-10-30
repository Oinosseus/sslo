pub mod emails;
pub mod users;
pub mod cookie_logins;

use std::error::Error;
use sqlx::sqlite::SqlitePool;
use crate::db::members::cookie_logins::Table;

#[derive(Clone)]
pub struct DbMembers {
    db_pool: SqlitePool,
    pub tbl_emails: emails::Table,
    pub tbl_users: users::Table,
    pub tbl_cookie_logins: Table,
}


impl DbMembers {

    pub fn new(db_pool: SqlitePool) -> Self {
        Self {
            db_pool: db_pool.clone(),
            tbl_emails: emails::Table::new(db_pool.clone()),
            tbl_users: users::Table::new(db_pool.clone()),
            tbl_cookie_logins: cookie_logins::Table::new(db_pool.clone()),
        }
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
