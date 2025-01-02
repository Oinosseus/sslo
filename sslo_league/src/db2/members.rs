pub mod users;
mod cookie_logins;

use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use sslo_lib::db::DatabaseError;
use users::{UserTableData, UserTableInterface};
use cookie_logins::CookieLoginTableData;
use crate::db2::members::cookie_logins::CookieLoginTableInterface;

/// The members database
pub struct MembersDbData {
    // pool: SqlitePool,
    tbl_users: Arc<RwLock<UserTableData>>,
    tbl_cookie_logins: Arc<RwLock<CookieLoginTableData>>
}

impl MembersDbData {
    /// When db_path is None, the pool is generated in memory
    pub(super) async fn new(db_path: Option<&Path>) -> Result<Arc<RwLock<Self>>, DatabaseError> {

        // set up db
        let pool = sslo_lib::db::get_pool(db_path);
        sqlx::migrate!("../rsc/db_migrations/league_members").run(&pool).await?;

        // create data object
        Ok(Arc::new_cyclic(|me| {
            RwLock::new(Self {
                // pool: pool.clone(),
                tbl_users: UserTableData::new(pool.clone()),
                tbl_cookie_logins: CookieLoginTableData::new(pool.clone(), me.clone()),
            })
        }))
    }
}

pub struct MembersDbInterface(Arc<RwLock<MembersDbData>>);

impl MembersDbInterface {

    pub(super) fn new(data: Arc<RwLock<MembersDbData>>) -> Self {
        Self(data)
    }

    pub async fn tbl_users(&self) -> UserTableInterface {
        let data = self.0.read().await;
        UserTableInterface::new(data.tbl_users.clone())
    }

    pub async fn tbl_cookie_logins(&self) -> CookieLoginTableInterface {
        let data = self.0.read().await;
        CookieLoginTableInterface::new(data.tbl_cookie_logins.clone())
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
            let cookie = tbl.from_id(1).await.unwrap();
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
            let item = tbl.from_cookie("unit test".to_string(), &cookie).await.unwrap();
            assert_eq!(item.user().await.unwrap().id().await, 1);
            assert!(item.last_usage().await.unwrap() >= now);
            assert_eq!(item.last_useragent().await.unwrap(), "unit test".to_string());
        }
    }
}