use std::collections::HashMap;
use std::sync::{Arc, RwLock, Weak};
use sqlx::SqlitePool;
use crate::db2::members::Members;
use sslo_lib::db::PoolPassing;
use super::item::Item;

pub struct Table {
    pool_ref_2me: Weak<dyn PoolPassing>,
    pool_ref_2parent: Weak<dyn PoolPassing>,
    items: RwLock<HashMap<i64, Arc<Item>>>,
}

impl Table {

    fn new(pool_ref: Weak<dyn PoolPassing>) -> Arc<Self> {
        Arc::new_cyclic(|me: &Weak<Self>| {
            Self {
                pool_ref_2me: me.clone(),
                pool_ref_2parent: pool_ref,
                items: RwLock::new(HashMap::new()),
            }
        })
    }

    pub fn name(&self) -> &'static str { super::tablename!() }


    /// Get an item
    /// This first tries to load the item from cache,
    /// and secondly load it from the database.
    async fn item_by_id(&self, id: i64) -> Option<Arc<Item>> {

        // cache hit
        match self.items.read() {
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
        match self.items.write() {
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

                // query
                let mut rows = match sqlx::query_as(concat!("SELECT rowid,* FROM ", tablename!(), " WHERE rowid = $1 LIMIT 2;"))
                    .bind(id)
                    .fetch_all(&pool)
                    .await {
                    Ok(r) => r,
                    Err(e) => {
                        log::error!("Failed to query database: {}", e);
                        return None;
                    }
                };

                // ambiguity check
                #[cfg(debug_assertions)]
                if rows.len() > 1 {
                    log::error!("Ambiguous rowid for db.members.users.rowid={}", id);
                    return None;
                }

                // get item
                let row = rows.pop()?;
                let item = Item::from_row(self.pool_ref_2me.clone(), row);

                // update cache
                cache.insert(item.id(), item.clone());

                // return
                return Some(item);
            }
        }
    }
}

impl PoolPassing for Table {
    fn pool(&self) -> Option<SqlitePool> {
        self.pool_ref_2parent.upgrade()?.pool()
    }
}


#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use sqlx::SqlitePool;
    use super::*;

    struct TestPoolPasser {
        pub pool: SqlitePool,
        pub pool_ref_2me: Weak<dyn PoolPassing>,
    }
    impl TestPoolPasser {
        fn new() -> Arc<Self> {
            let sqlite_opts = SqliteConnectOptions::from_str(":memory:").unwrap();
            let pool = SqlitePoolOptions::new()
                .min_connections(1)
                .max_connections(1)  // default is 10
                .idle_timeout(None)
                .max_lifetime(None)
                .connect_lazy_with(sqlite_opts);
            Arc::new_cyclic(|me: &Weak<Self>| {
                Self{pool, pool_ref_2me: me.clone()}
            })
        }
        async fn init(&self) {
            sqlx::migrate!("../rsc/db_migrations/league_members").run(&self.pool).await.unwrap();
            sqlx::query(concat!("INSERT INTO ", super::super::tablename!(), " (name, email) VALUES ($1, $2);"))
                .bind("username")
                .bind("user@email.tld")
                .execute(&self.pool)
                .await.unwrap();
        }
    }
    impl PoolPassing for TestPoolPasser {
        fn pool(&self) -> Option<SqlitePool> {Some(self.pool.clone())}
    }

    #[tokio::test]
    async fn test_item() {

        // create test table
        let pool_passer = TestPoolPasser::new();
        pool_passer.init().await;
        let ref_pool_passer: Arc<dyn PoolPassing> = pool_passer.clone();
        let tbl = Table::new(Arc::downgrade(&ref_pool_passer));

        // test failed retrieval
        let i = tbl.item_by_id(999).await;
        assert!(i.is_none());

        // test retrieval
        let i = tbl.item_by_id(1).await.unwrap();
        assert_eq!(i.id(), 1);
    }
}