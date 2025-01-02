mod users;
mod cookie_logins;

use std::sync::Arc;
use sqlx::SqlitePool;
use tokio::sync::RwLock;
use sslo_lib::db::DatabaseError;
use users::{UserTableData, UserTableInterface};

/// The members database
pub struct MembersDbData {
    pool: SqlitePool,
    tbl_users: Arc<RwLock<UserTableData>>,
}

impl MembersDbData {

    /// When db_path is None, the pool is generated in memory
    pub(super) async fn new(db_path: Option<&str>) -> Result<Arc<RwLock<Self>>, DatabaseError> {

        // set up db
        let pool = sslo_lib::db::get_pool(db_path);
        sqlx::migrate!("../rsc/db_migrations/league_members").run(&pool).await?;

        // set up tables
        let tbl_users = UserTableData::new(&pool);

        // create data object
        Ok(Arc::new(RwLock::new(Self {
            pool,
            tbl_users,
        })))
    }
}

struct MembersDbInterface(Arc<RwLock<MembersDbData>>);

impl MembersDbInterface {

    pub(super) fn new(data: Arc<RwLock<MembersDbData>>) -> Self {
        Self(data)
    }

    pub async fn tbl_users(&self) -> users::UserTableInterface {
        let data = self.0.read().await;
        UserTableInterface::new(data.tbl_users.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;

    #[test(tokio::test)]
    async fn create_new() {
        let data = MembersDbData::new(None).await.unwrap();
        let db = MembersDbInterface::new(data);
        let _ = db.tbl_users().await;
    }
}