pub mod users;
pub mod cookie_logins;
pub mod steam_users;
mod email_accounts;

use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use users::{UserTableData, UserTable};
use cookie_logins::CookieLoginTableData;
use sslo_lib::error::SsloError;
use crate::db2::members::cookie_logins::CookieLoginTable;
use crate::db2::members::email_accounts::{EmailAccountsTable, EmailAccountsTableData};
use crate::db2::members::steam_users::{SteamUserTable, SteamUserTableData};

/// The members database
pub struct MembersDbData {
    // pool: SqlitePool,
    tbl_users: Arc<RwLock<UserTableData>>,
    tbl_cookie_logins: Arc<RwLock<CookieLoginTableData>>,
    tbl_steam_user: Arc<RwLock<SteamUserTableData>>,
    tbl_email_accounts: Arc<RwLock<EmailAccountsTableData>>,
}

impl MembersDbData {
    /// When db_path is None, the pool is generated in memory
    pub(super) async fn new(db_path: Option<&Path>) -> Result<Arc<RwLock<Self>>, SsloError> {

        // set up db_obsolete
        let pool = sslo_lib::db::get_pool(db_path);
        sqlx::migrate!("../rsc/db_migrations/league_members").run(&pool).await?;

        // create data object
        Ok(Arc::new_cyclic(|me| {
            RwLock::new(Self {
                // pool: pool.clone(),
                tbl_users: UserTableData::new(pool.clone()),
                tbl_cookie_logins: CookieLoginTableData::new(pool.clone(), me.clone()),
                tbl_steam_user: SteamUserTableData::new(pool.clone(), me.clone()),
                tbl_email_accounts: EmailAccountsTableData::new(pool.clone(), me.clone()),
            })
        }))
    }
}

pub struct MembersDbInterface(Arc<RwLock<MembersDbData>>);

impl MembersDbInterface {

    pub(super) fn new(data: Arc<RwLock<MembersDbData>>) -> Self {
        Self(data)
    }

    pub async fn tbl_users(&self) -> UserTable {
        let data = self.0.read().await;
        UserTable::new(data.tbl_users.clone())
    }

    pub async fn tbl_cookie_logins(&self) -> CookieLoginTable {
        let data = self.0.read().await;
        CookieLoginTable::new(data.tbl_cookie_logins.clone())
    }

    pub async fn tbl_steam_user(&self) -> SteamUserTable {
        let data = self.0.read().await;
        SteamUserTable::new(data.tbl_steam_user.clone())
    }

    pub async fn tbl_email_accounts(&self) -> EmailAccountsTable {
        let data = self.0.read().await;
        EmailAccountsTable::new(data.tbl_email_accounts.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;

    async fn get_db() -> MembersDbInterface {
        let data = MembersDbData::new(None).await.unwrap();
        MembersDbInterface::new(data)
    }

    #[test(tokio::test)]
    async fn create_new() {
        let data = MembersDbData::new(None).await.unwrap();
        let db = MembersDbInterface::new(data);
        let _ = db.tbl_users().await;
    }

    mod cookie_logins {
        use chrono::Utc;
        use test_log::test;

        #[test(tokio::test)]
        async fn create_new_cookie() {
            let db = super::get_db().await;
            let user = db.tbl_users().await.create_new_user().await.unwrap();
            let mut tbl = db.tbl_cookie_logins().await;
            let cookie = tbl.create_new_cookie(&user).await.unwrap();
            assert_eq!(cookie.id().await, 1);
            assert!(cookie.get_cookie().await.is_some());
            assert!(cookie.get_cookie().await.is_none());

            // retrieve from cache
            let cookie = tbl.item_by_id(1).await.unwrap();
            assert_eq!(cookie.id().await, 1);
            assert!(cookie.get_cookie().await.is_none());
        }

        #[test(tokio::test)]
        async fn cookie_flow() {
            let now = Utc::now();
            let db = super::get_db().await;
            let mut tbl = db.tbl_cookie_logins().await;

            // create new cookie
            let user = db.tbl_users().await.create_new_user().await.unwrap();
            assert_eq!(user.id().await, 1);
            let item = tbl.create_new_cookie(&user).await.unwrap();
            let cookie = item.get_cookie().await.unwrap();

            // try validate cookie
            let item = tbl.item_by_cookie("unit test".to_string(), &cookie).await.unwrap();
            assert_eq!(item.user().await.unwrap().id().await, 1);
            assert!(item.last_usage().await.unwrap() >= now);
            assert_eq!(item.last_useragent().await.unwrap(), "unit test".to_string());
        }

        #[test(tokio::test)]
        async fn item_from_last_usage() {
            let now = Utc::now();
            let db = super::get_db().await;
            let mut tbl = db.tbl_cookie_logins().await;
            let user = db.tbl_users().await.create_new_user().await.unwrap();

            // create first login cookie
            let item1 = tbl.create_new_cookie(&user).await.unwrap();
            let cookie1 = item1.get_cookie().await.unwrap();

            // create second login cookie
            let item2 = tbl.create_new_cookie(&user).await.unwrap();
            let cookie2 = item2.get_cookie().await.unwrap();

            // use cookie 2 and then, later cookie 1
            tbl.item_by_cookie("unit test".to_string(), &cookie2).await.unwrap();
            tokio::time::sleep(std::time::Duration::from_millis(1100)).await;  // wait a second
            tbl.item_by_cookie("unit test".to_string(), &cookie1).await.unwrap();

            // check that cookie1 is the latest
            let item = tbl.item_from_latest_usage(&user).await.unwrap();
            assert_eq!(item.id().await, item1.id().await);

            // use cookie 2 again and check for latest
            tokio::time::sleep(std::time::Duration::from_millis(1100)).await;  // wait a second
            tbl.item_by_cookie("unit test".to_string(), &cookie2).await.unwrap();
            let item = tbl.item_from_latest_usage(&user).await.unwrap();
            assert_eq!(item.id().await, item2.id().await);
        }
    }

    mod email_accounts {
        use test_log::test;
        use super::*;

        #[test(tokio::test)]
        async fn set_get_user() {
            let db = get_db().await;

            // set user
            {
                let usr = db.tbl_users().await.create_new_user().await.unwrap();
                assert_eq!(usr.id().await, 1);
                let eml = db.tbl_email_accounts().await.create_account("a.b@c.de".to_string()).await.unwrap();
                assert!(eml.set_user(&usr).await);

                // get user
                assert_eq!(eml.user().await.unwrap().id().await, usr.id().await);
            }
        }

    }
}