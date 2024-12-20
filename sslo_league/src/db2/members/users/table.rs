use std::collections::HashMap;
use std::sync::{Arc, RwLock, Weak};
use sqlx::SqlitePool;
use crate::db2::members::Members;
use sslo_lib::db::PoolPassing;

pub struct Table {
    db: RwLock<Weak<Members>>,
    items: HashMap<i64, Arc<super::item::Item>>,
}

impl Table {

    fn new() -> Arc<Self> {
        Arc::new(Self{
            db: RwLock::new(Weak::new()),
            items: HashMap::new(),
        })
    }

    pub fn name(&self) -> &'static str { super::tablename!() }

}

impl PoolPassing for Table {
    fn pool(&self) -> Option<SqlitePool> {
        match self.db.read() {
            Err(_) => {
                log::error!("Could not read database connection!");
                None
            },
            Ok(guard) => {
                Some(guard.upgrade()?.pool())
            }
        }
    }
}