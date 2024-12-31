mod row;
mod item;

/// This is the central defined name of the table in this module,
/// used to allow copy&paste of the code for other tables.
macro_rules! tablename {
    () => { "users" };
}

use std::collections::HashMap;
use std::sync::Arc;
pub(self) use tablename;

use sqlx::SqlitePool;
use tokio::sync::RwLock;

struct TableData {
    pool: SqlitePool,
    item_cache: HashMap<i64, Arc<RwLock<item::ItemData>>>
}

impl TableData {
    pub(super) fn new(pool: &SqlitePool) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            pool: pool.clone(),
            item_cache: HashMap::new(),
        }))
    }
}

struct TableInterface (
    Arc<RwLock<TableData>>
);

impl TableInterface {

    pub fn new(data: Arc<RwLock<TableData>>) -> Self {
        Self(data)
    }

    /// Create a new user
    pub async fn new_item(&self) -> Option<item::ItemInterface> {

        // insert new item into DB
        let mut row = row::ItemDbRow::new(0);
        {
            let pool = self.0.read().await.pool.clone();
            match row.store(&pool).await {
                Ok(_) => {}
                Err(e) => {
                    log::error!("Could not create a new user: {}", e);
                    return None;
                }
            }
        }

        // update cache
        let mut tbl_data = self.0.write().await;
        let item_data = item::ItemData::new(&tbl_data.pool, row);
        let item = item::ItemInterface::new(item_data.clone());
        tbl_data.item_cache.insert(item.id().await, item_data);
        return Some(item);
    }

    /// Get an item
    /// This first tries to load the item from cache,
    /// and secondly load it from the database.
    pub async fn item_by_id(&self, id: i64) -> Option<item::ItemInterface> {

        // try cache hit
        {
            let tbl_data = self.0.read().await;
            if let Some(item_data) = tbl_data.item_cache.get(&id) {
                return Some(item::ItemInterface::new(item_data.clone()));
            }
        }

        // try loading from DB if not found in cache
        {
            let mut tbl_data = self.0.write().await;

            // load from db
            let mut row = row::ItemDbRow::new(id);
            match row.load(&tbl_data.pool).await {
                Ok(_) => { },
                Err(e) => {
                    if e.is_rowid_not_found() {
                        log::warn!("{}", e);
                    } else {
                        log::error!("{}", e.to_string());
                    }
                    return None;
                },
            }
            debug_assert_eq!(row.rowid, id);

            // create item
            let item_data = item::ItemData::new(&tbl_data.pool, row);
            let item = item::ItemInterface::new(item_data.clone());
            tbl_data.item_cache.insert(id, item_data);
            return Some(item);
        }
    }


    // /// Search the database for an email and then return the item
    // /// The search is case-insensitive
    // pub async fn item_by_email(&self, email: &str) -> Option<Arc<Item>> {
    //
    //     // get pool
    //     let pool = match self.pool() {
    //         Some(pool) => pool,
    //         None => {
    //             log::error!("No pool!");
    //             return None;
    //         }
    //     };
    //
    //     // find item in database
    //     let row = Row::from_db_by_email(&pool, email).await.or_else(||{
    //         log::warn!("No item with email={} found!", email);
    //         None
    //     })?;
    //     self.item_by_id(row.rowid).await
    // }

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

    async fn get_table_interface() -> TableInterface {
        let pool = get_pool().await;
        let tbl_data = TableData::new(&pool);
        TableInterface::new(tbl_data.clone())
    }

    #[test(tokio::test)]
    async fn new_item() {
        let tbl = get_table_interface().await;
        assert_eq!(tbl.0.read().await.item_cache.len(), 0);
        let item = tbl.new_item().await.unwrap();
        assert_eq!(tbl.0.read().await.item_cache.len(), 1);
        assert_eq!(item.id().await, 1);
    }

    #[test(tokio::test)]
    async fn item_by_id_from_db() {
        let tbl = get_table_interface().await;

        // check if cache is empty
        {
            let cache = tbl.0.read().await;
            assert_eq!(cache.item_cache.len(), 0);
        }

        // append items to db
        let mut item = tbl.new_item().await.unwrap();
        item.set_name("Bob".to_string()).await.unwrap();
        let mut item = tbl.new_item().await.unwrap();
        item.set_name("Dylan".to_string()).await.unwrap();

        // check if cache is filled
        {
            let cache = tbl.0.read().await;
            assert_eq!(cache.item_cache.len(), 2);
        }

        // retrieve item
        let item1 = tbl.item_by_id(1).await.unwrap();
        assert_eq!(item1.id().await, 1);
        assert_eq!(item1.name().await, "Bob");
        let item2 = tbl.item_by_id(2).await.unwrap();
        assert_eq!(item2.id().await, 2);
        assert_eq!(item2.name().await, "Dylan");
    }
}