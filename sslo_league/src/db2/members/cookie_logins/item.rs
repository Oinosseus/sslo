use std::sync::Arc;
use chrono::{DateTime, Utc};
use rand::RngCore;
use tokio::sync::RwLock;
use sqlx::SqlitePool;
use super::row::ItemDbRow;
use sslo_lib::db::DatabaseError;
use sslo_lib::token;

/// The actual data of an item that is shared by Arc<RwLock<ItemData>>
pub(super) struct ItemData {
    pool: SqlitePool,
    row: ItemDbRow,
}

impl ItemData {
    pub fn new(pool: &SqlitePool, row: ItemDbRow) -> Arc<RwLock<ItemData>> {
        Arc::new(RwLock::new(Self {
            pool: pool.clone(),
            row,
        }))
    }
}

/// This abstracts data access to shared items
pub struct ItemInterface(Arc<RwLock<ItemData>>);

impl ItemInterface {
    /// Set up an object from shared data (assumed to be retrieved from database)
    pub(super) fn new(item_data: Arc<RwLock<ItemData>>) -> Self {
        Self(item_data)
    }

    pub async fn id(&self) -> i64 {
        self.0.read().await.row.rowid
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use sqlx::SqlitePool;
    use super::*;
    use test_log::test;

    async fn get_pool() -> SqlitePool {
        let sqlite_opts = SqliteConnectOptions::from_str(":memory:").unwrap();
        let pool = SqlitePoolOptions::new()
            .min_connections(1)
            .max_connections(1)  // default is 10
            .idle_timeout(None)
            .max_lifetime(None)
            .connect_lazy_with(sqlite_opts);
        sqlx::migrate!("../rsc/db_migrations/league_members").run(&pool).await.unwrap();
        return pool;
    }

    async fn create_new_item(pool: &SqlitePool) -> ItemInterface {
        let row = ItemDbRow::new(0);
        let data = ItemData::new(pool, row);
        ItemInterface::new(data)
    }
}
