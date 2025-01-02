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
use item::UserInterface;

pub(super) struct UserTableData {
    pool: SqlitePool,
    item_cache: HashMap<i64, Arc<RwLock<item::UserData>>>
}

impl UserTableData {
    pub(super) fn new(pool: &SqlitePool) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            pool: pool.clone(),
            item_cache: HashMap::new(),
        }))
    }
}

pub struct UserTableInterface(
    Arc<RwLock<UserTableData>>
);

impl UserTableInterface {

    pub fn new(data: Arc<RwLock<UserTableData>>) -> Self {
        Self(data)
    }

    /// Create a new user
    pub async fn create_new_user(&self) -> Option<UserInterface> {

        // insert new item into DB
        let mut row = row::UserDbRow::new(0);
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
        let item_data = item::UserData::new(&tbl_data.pool, row);
        let item = UserInterface::new(item_data.clone());
        tbl_data.item_cache.insert(item.id().await, item_data);
        return Some(item);
    }

    /// Get an item
    /// This first tries to load the item from cache,
    /// and secondly load it from the database.
    pub async fn user_by_id(&self, id: i64) -> Option<UserInterface> {

        // try cache hit
        {
            let tbl_data = self.0.read().await;
            if let Some(item_data) = tbl_data.item_cache.get(&id) {
                return Some(UserInterface::new(item_data.clone()));
            }
        }

        // try loading from DB if not found in cache
        {
            let mut tbl_data = self.0.write().await;

            // load from db
            let mut row = row::UserDbRow::new(id);
            match row.load(&tbl_data.pool).await {
                Ok(_) => { },
                Err(e) => {
                    if e.is_not_found_type() {
                        log::warn!("{}", e);
                    } else {
                        log::error!("{}", e.to_string());
                    }
                    return None;
                },
            }
            debug_assert_eq!(row.rowid, id);

            // create item
            let item_data = item::UserData::new(&tbl_data.pool, row);
            let item = item::UserInterface::new(item_data.clone());
            tbl_data.item_cache.insert(id, item_data);
            return Some(item);
        }
    }


    /// Search the database for an email and then return the item
    /// The search is case-insensitive,
    /// this is not cached -> expensive
    pub async fn user_by_email(&self, email: &str) -> Option<UserInterface> {
        let pool: SqlitePool;
        {   // scoped lock to call user_by_id() later
            let data = self.0.read().await;
            pool = data.pool.clone();
        }
        let row = match row::UserDbRow::from_email(email, &pool).await {
            Ok(row) => {row},
            Err(e) => {
                if e.is_not_found_type() {
                    log::warn!("{}", e);
                } else {
                    log::error!("{}", e.to_string());
                }
                return None;
            },
        };
        return self.user_by_id(row.rowid).await;
    }

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

    async fn get_table_interface() -> UserTableInterface {
        let pool = get_pool().await;
        let tbl_data = UserTableData::new(&pool);
        UserTableInterface::new(tbl_data.clone())
    }

    #[test(tokio::test)]
    async fn create_new_user() {
        let tbl = get_table_interface().await;
        assert_eq!(tbl.0.read().await.item_cache.len(), 0);
        let item = tbl.create_new_user().await.unwrap();
        assert_eq!(tbl.0.read().await.item_cache.len(), 1);
        assert_eq!(item.id().await, 1);
    }

    #[test(tokio::test)]
    async fn user_by_id() {
        let tbl = get_table_interface().await;

        // check if cache is empty
        {
            let cache = tbl.0.read().await;
            assert_eq!(cache.item_cache.len(), 0);
        }

        // append items to db
        let mut item = tbl.create_new_user().await.unwrap();
        item.set_name("Bob".to_string()).await.unwrap();
        let mut item = tbl.create_new_user().await.unwrap();
        item.set_name("Dylan".to_string()).await.unwrap();

        // check if cache is filled
        {
            let cache = tbl.0.read().await;
            assert_eq!(cache.item_cache.len(), 2);
        }

        // retrieve item
        let item1 = tbl.user_by_id(1).await.unwrap();
        assert_eq!(item1.id().await, 1);
        assert_eq!(item1.name().await, "Bob");
        let item2 = tbl.user_by_id(2).await.unwrap();
        assert_eq!(item2.id().await, 2);
        assert_eq!(item2.name().await, "Dylan");
    }

    #[test(tokio::test)]
    async fn user_by_email() {
        let tbl = get_table_interface().await;
        let mut user = tbl.create_new_user().await.unwrap();
        let token = user.set_email("a.B@c.de".to_string()).await.unwrap();
        assert!(user.verify_email(token).await);

        // retrieve item
        let user = tbl.user_by_email("a.B@c.de").await.unwrap();
        assert_eq!(user.email().await.unwrap(), "a.B@c.de".to_string());

        // check case insensitivity
        let user = tbl.user_by_email("a.b@c.de").await.unwrap();
        assert_eq!(user.email().await.unwrap(), "a.B@c.de".to_string());
    }
}