mod row;
mod item;

macro_rules! tablename {
    {} => { "steam_users" };
}

use std::collections::HashMap;
use std::sync::{Arc, Weak};
use sqlx::SqlitePool;
use tokio::sync::RwLock;
pub(self) use tablename;
use crate::db2::members::MembersDbData;

pub(super) struct SteamUserTableData {
    pool: SqlitePool,
    item_cache_by_rowid: HashMap<i64, Arc<RwLock<item::SteamUserData>>>,
    item_cache_by_steamid: HashMap<String, Arc<RwLock<item::SteamUserData>>>,
    db_members: Weak<RwLock<MembersDbData>>,
}

impl SteamUserTableData {
    pub(super) fn new(pool: SqlitePool, db_members: Weak<RwLock<MembersDbData>>) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            pool,
            item_cache_by_rowid: HashMap::new(),
            item_cache_by_steamid: HashMap::new(),
            db_members,
        }))
    }
}

pub struct SteamUserTableInterface (Arc<RwLock<SteamUserTableData>>);

impl SteamUserTableInterface {
    pub(super) fn new(data: Arc<RwLock<SteamUserTableData>>) -> Self { Self(data) }
}
