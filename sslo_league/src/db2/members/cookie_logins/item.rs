use std::sync::Arc;
use chrono::{DateTime, Utc};
use rand::RngCore;
use tokio::sync::RwLock;
use sqlx::SqlitePool;
use super::row::CookieLoginDbRow;
use sslo_lib::db::DatabaseError;
use sslo_lib::token;

/// The actual data of an item that is shared by Arc<RwLock<ItemData>>
pub(super) struct CookieLoginData {
    pool: SqlitePool,
    row: CookieLoginDbRow,
}

impl CookieLoginData {
    pub fn new(pool: &SqlitePool, row: CookieLoginDbRow) -> Arc<RwLock<CookieLoginData>> {
        Arc::new(RwLock::new(Self {
            pool: pool.clone(),
            row,
        }))
    }
}

/// This abstracts data access to shared items
pub struct CookieLoginInterface(Arc<RwLock<CookieLoginData>>);

impl CookieLoginInterface {
    /// Set up an object from shared data (assumed to be retrieved from database)
    pub(super) fn new(item_data: Arc<RwLock<CookieLoginData>>) -> Self {
        Self(item_data)
    }

    // pub async fn id(&self) -> i64 {
    //     self.0.read().await.row.rowid
    // }
}

#[cfg(test)]
mod tests {
    use sqlx::SqlitePool;
    use super::*;
    use test_log::test;

    async fn get_pool() -> SqlitePool {
        let pool = sslo_lib::db::get_pool(None);
        sqlx::migrate!("../rsc/db_migrations/league_members").run(&pool).await.unwrap();
        return pool;
    }

    #[test(tokio::test)]
    async fn new() {
        let pool = get_pool().await;
        let row = CookieLoginDbRow::new(0);
        let data = CookieLoginData::new(&pool, row);
        CookieLoginInterface::new(data);
    }
}
