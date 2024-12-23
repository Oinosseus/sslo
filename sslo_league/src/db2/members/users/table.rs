use std::collections::HashMap;
use std::sync::{Arc, RwLock, Weak};
use sqlx::SqlitePool;
use sslo_lib::db::PoolPassing;
use crate::db2::members::users::row::Row;
use super::item::Item;

pub struct Table {
    pool_ref_2me: Weak<dyn PoolPassing>,
    pool_ref_2parent: Weak<dyn PoolPassing>,
    item_cache: RwLock<HashMap<i64, Arc<Item>>>,
}

impl Table {

    pub(super) fn new(pool_ref: Weak<dyn PoolPassing>) -> Arc<Self> {
        Arc::new_cyclic(|me: &Weak<Self>| {
            Self {
                pool_ref_2me: me.clone(),
                pool_ref_2parent: pool_ref,
                item_cache: RwLock::new(HashMap::new()),
            }
        })
    }

    pub fn name(&self) -> &'static str { super::tablename!() }


    /// Get an item
    /// This first tries to load the item from cache,
    /// and secondly load it from the database.
    pub async fn item_by_id(&self, id: i64) -> Option<Arc<Item>> {

        // cache hit
        match self.item_cache.read() {
            Err(e) => {
                log::error!("Failed to read item cache: {}", e);
                return None;
            },
            Ok(cache) => {
                if let Some(item) = cache.get(&id) {
                    return Some(item.clone());
                }
            }
        }

        // try loading from DB if not found in cache
        match self.item_cache.write() {
            Err(e) => {
                log::error!("Failed to write-lock cache: {}", e);
                return None;
            },
            Ok(mut cache) => {

                // get pool
                let pool = match self.pool() {
                    Some(pool) => pool,
                    None => {
                        log::error!("No pool!");
                        return None;
                    }
                };

                // request item from db
                let row = Row::from_db_by_id(&pool, id).await.or_else(||{
                    log::warn!("No db item with id={} found!", id);
                    None
                })?;
                let item = Item::from_row(self.pool_ref_2me.clone(), row);

                // update cache
                cache.insert(item.id(), item.clone());

                // return
                Some(item)
            }
        }
    }


    /// Search the database for an email and then return the item
    /// The search is case-insensitive
    pub async fn item_by_email(&self, email: &str) -> Option<Arc<Item>> {

        // get pool
        let pool = match self.pool() {
            Some(pool) => pool,
            None => {
                log::error!("No pool!");
                return None;
            }
        };

        // find item in database
        let row = Row::from_db_by_email(&pool, email).await.or_else(||{
            log::warn!("No item with email={} found!", email);
            None
        })?;
        self.item_by_id(row.rowid).await
    }
}

impl PoolPassing for Table {
    fn pool(&self) -> Option<SqlitePool> {
        self.pool_ref_2parent.upgrade()?.pool()
    }
}
