mod row;
mod item;

/// This is the central defined name of the table in this module,
/// used to allow copy&paste of the code for other tables.
macro_rules! tablename {
    () => { "cookie_logins" };
}

use std::collections::HashMap;
use std::sync::Arc;
use sqlx::SqlitePool;
use tokio::sync::RwLock;
pub(self) use tablename;

pub(super) struct CookieLoginTableData {
    pool: SqlitePool,
    item_cache: HashMap<i64, Arc<RwLock<item::CookieLoginData>>>,
}

impl CookieLoginTableData {
    pub fn new(pool: SqlitePool) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            pool,
            item_cache: HashMap::new(),
        }))
    }
}

struct CookieLoginInterface(Arc<RwLock<item::CookieLoginData>>);

impl CookieLoginInterface {
    pub fn new(data: Arc<RwLock<item::CookieLoginData>>) -> Self { Self(data) }
}