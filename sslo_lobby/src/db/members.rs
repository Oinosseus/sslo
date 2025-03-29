mod users;

use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use users::{UserTableData, UserTable};
// use cookie_logins::CookieLoginTableData;
use sslo_lib::error::SsloError;
// use members::cookie_logins::CookieLoginTable;
// use members::email_accounts::{EmailAccountsTable, EmailAccountsTableData};
// use members::steam_accounts::{SteamAccountsTable, SteamAccountsTableData};

/// The members database
pub struct MembersDbData {
    // pool: SqlitePool,
    tbl_users: Arc<RwLock<UserTableData>>,
    // tbl_cookie_logins: Arc<RwLock<CookieLoginTableData>>,
    // tbl_steam_accounts: Arc<RwLock<SteamAccountsTableData>>,
    // tbl_email_accounts: Arc<RwLock<EmailAccountsTableData>>,
}

impl MembersDbData {
    /// When db_path is None, the pool is generated in memory
    pub(super) async fn new(db_path: Option<&Path>) -> Result<Arc<RwLock<Self>>, SsloError> {

        // set up db_obsolete
        let pool = sslo_lib::db::get_pool(db_path);
        sqlx::migrate!("../rsc/db_migrations/lobby_members").run(&pool).await?;

        // create data object
        Ok(Arc::new_cyclic(|me| {
            RwLock::new(Self {
                // pool: pool.clone(),
                tbl_users: UserTableData::new(pool.clone()),
                // tbl_cookie_logins: CookieLoginTableData::new(pool.clone(), me.clone()),
                // tbl_steam_accounts: SteamAccountsTableData::new(pool.clone(), me.clone()),
                // tbl_email_accounts: EmailAccountsTableData::new(pool.clone(), me.clone()),
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

    // pub async fn tbl_cookie_logins(&self) -> CookieLoginTable {
    //     let data = self.0.read().await;
    //     CookieLoginTable::new(data.tbl_cookie_logins.clone())
    // }

    // pub async fn tbl_steam_accounts(&self) -> SteamAccountsTable {
    //     let data = self.0.read().await;
    //     SteamAccountsTable::new(data.tbl_steam_accounts.clone())
    // }
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
    }

    // mod cookie_logins {
    //     use chrono::Utc;
    //     use test_log::test;
    //
    //     #[test(tokio::test)]
    //     async fn create_new_cookie() {
    //         let db = super::get_db().await;
    //         let user = db.tbl_users().await.create_new_user().await.unwrap();
    //         let mut tbl = db.tbl_cookie_logins().await;
    //         let cookie = tbl.create_new_cookie(&user).await.unwrap();
    //         assert_eq!(cookie.id().await, 1);
    //         assert!(cookie.get_cookie().await.is_some());
    //         assert!(cookie.get_cookie().await.is_none());
    //
    //         // retrieve from cache
    //         let cookie = tbl.item_by_id(1).await.unwrap();
    //         assert_eq!(cookie.id().await, 1);
    //         assert!(cookie.get_cookie().await.is_none());
    //     }
    //
    //     #[test(tokio::test)]
    //     async fn cookie_flow() {
    //         let now = Utc::now();
    //         let db = super::get_db().await;
    //         let mut tbl = db.tbl_cookie_logins().await;
    //
    //         // create new cookie
    //         let user = db.tbl_users().await.create_new_user().await.unwrap();
    //         assert_eq!(user.id().await, 1);
    //         let item = tbl.create_new_cookie(&user).await.unwrap();
    //         let cookie = item.get_cookie().await.unwrap();
    //
    //         // try validate cookie
    //         let item = tbl.item_by_cookie("unit test".to_string(), &cookie).await.unwrap();
    //         assert_eq!(item.user().await.unwrap().id().await, 1);
    //         assert!(item.last_usage().await.unwrap() >= now);
    //         assert_eq!(item.last_useragent().await.unwrap(), "unit test".to_string());
    //     }
    //
    //     #[test(tokio::test)]
    //     async fn item_from_last_usage() {
    //         let now = Utc::now();
    //         let db = super::get_db().await;
    //         let mut tbl = db.tbl_cookie_logins().await;
    //         let user = db.tbl_users().await.create_new_user().await.unwrap();
    //
    //         // create first login cookie
    //         let item1 = tbl.create_new_cookie(&user).await.unwrap();
    //         let cookie1 = item1.get_cookie().await.unwrap();
    //
    //         // create second login cookie
    //         let item2 = tbl.create_new_cookie(&user).await.unwrap();
    //         let cookie2 = item2.get_cookie().await.unwrap();
    //
    //         // use cookie 2 and then, later cookie 1
    //         tbl.item_by_cookie("unit test".to_string(), &cookie2).await.unwrap();
    //         tokio::time::sleep(std::time::Duration::from_millis(1100)).await;  // wait a second
    //         tbl.item_by_cookie("unit test".to_string(), &cookie1).await.unwrap();
    //
    //         // check that cookie1 is the latest
    //         let item = tbl.item_from_latest_usage(&user).await.unwrap();
    //         assert_eq!(item.id().await, item1.id().await);
    //
    //         // use cookie 2 again and check for latest
    //         tokio::time::sleep(std::time::Duration::from_millis(1100)).await;  // wait a second
    //         tbl.item_by_cookie("unit test".to_string(), &cookie2).await.unwrap();
    //         let item = tbl.item_from_latest_usage(&user).await.unwrap();
    //         assert_eq!(item.id().await, item2.id().await);
    //     }
    // }

    // mod email_accounts {
    //     use test_log::test;
    //     use super::*;
    //
    //     #[test(tokio::test)]
    //     async fn items_by_user() {
    //         let db = get_db().await;
    //         let tbl_eml = db.tbl_email_accounts().await;
    //
    //         // create user
    //         let usr = db.tbl_users().await.create_new_user().await.unwrap();
    //         assert_eq!(usr.id().await, 1);
    //
    //         // creates email accounts
    //         let eml = tbl_eml.create_account("a.b@c.de".to_string()).await.unwrap();
    //         let token = eml.create_token(Some(&usr)).await.unwrap();
    //         assert!(eml.consume_token(token).await);
    //         let eml = tbl_eml.create_account("foo.bar@c.de".to_string()).await.unwrap();
    //         let token = eml.create_token(Some(&usr)).await.unwrap();
    //         assert!(eml.consume_token(token).await);
    //         let eml = tbl_eml.create_account("not.associated@elsewhere.net".to_string()).await.unwrap();
    //         let eml = tbl_eml.create_account("foo.baz@c.de".to_string()).await.unwrap();
    //         let token = eml.create_token(Some(&usr)).await.unwrap();
    //         assert!(eml.consume_token(token).await);
    //
    //         // check associated accounts
    //         let items = tbl_eml.items_by_user(&usr).await;
    //         assert_eq!(items.len(), 3);
    //     }
    // }

    // mod steam_accounts {
    //     use chrono::Utc;
    //     use test_log::test;
    //     use super::*;
    //
    //     #[test(tokio::test)]
    //     async fn login_procedure() {
    //         let db = get_db().await;
    //         let tbl_usr = db.tbl_users().await;
    //         let tbl_stm = db.tbl_steam_accounts().await;
    //
    //         // receive a new steam login
    //         let steam_id = "SteamId1";
    //
    //         // get steam account
    //         let steam_account = tbl_stm.item_by_steam_id(steam_id, true).await.unwrap();
    //         steam_account.set_last_login(Utc::now()).await.unwrap();
    //
    //         // get user
    //         let user = match steam_account.user().await {
    //             Some(user) => user,
    //             None => tbl_usr.create_new_user().await.unwrap()
    //         };
    //
    //         // check identical user at new login
    //         let steam_account = tbl_stm.item_by_steam_id(steam_id, false).await.unwrap();
    //         let user2 = steam_account.user().await.unwrap();
    //         assert_eq!(user.id().await, user2.id().await);
    //     }
    // }
}