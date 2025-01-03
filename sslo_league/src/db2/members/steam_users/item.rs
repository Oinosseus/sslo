use std::sync::{Arc, Weak};
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use tokio::sync::RwLock;
use super::super::{MembersDbInterface, MembersDbData};
use super::super::users::UserInterface;

/// The actual data of an item that is shared by Arc<RwLock<ItemData>>
pub(super) struct SteamUserData {
    pool: SqlitePool,
    row: super::row::SteamUserDbRow,
    db_members: Weak<RwLock<MembersDbData>>,
}

impl SteamUserData {
    pub fn new(pool: &SqlitePool, row: super::row::SteamUserDbRow, db_members: Weak<RwLock<MembersDbData>>) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            pool: pool.clone(),
            row,
            db_members,
        }))
    }
}

struct SteamUserLastLogin {
    time: DateTime<Utc>,
    useragent: String,
}

/// This abstracts data access to shared items
pub struct SteamUserInterface(Arc<RwLock<SteamUserData>>);

impl SteamUserInterface {

    pub(super) fn new(item_data: Arc<RwLock<SteamUserData>>) -> Self {
        Self(item_data)
    }

    pub async fn id(&self) -> i64 { self.0.read().await.row.rowid }
    pub async fn steam_id(&self) -> String { self.0.read().await.row.steam_id.clone() }
    pub async fn creation(&self) -> DateTime<Utc> { self.0.read().await.row.creation.clone() }

    pub async fn user(&self) -> Option<UserInterface> {
        let data = self.0.read().await;
        let db_members = match data.db_members.upgrade() {
            Some(db_data) => MembersDbInterface::new(db_data),
            None => {
                log::error!("cannot upgrade weak pointer for rowid={}, user={}", data.row.rowid, data.row.user);
                return None;
            }
        };
        db_members.tbl_users().await.user_by_id(data.row.rowid).await
    }

    pub async fn last_login(&self) -> Option<SteamUserLastLogin> {
        let data = self.0.read().await;
        let time = data.row.last_login_timestamp.clone()?;
        let useragent = data.row.last_login_useragent.clone()?;
        Some(SteamUserLastLogin { time, useragent })
    }
}