use std::sync::{Arc, RwLock, Weak};
use sqlx::SqlitePool;
use crate::db2::members::Members;

pub struct Table {
    db: RwLock<Weak<Members>>
}

impl Table {

    fn new() -> Arc<Self> {
        Arc::new(Self{
            db: RwLock::new(Weak::new()),
        })
    }

    pub fn name(&self) -> &'static str { "users" }

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