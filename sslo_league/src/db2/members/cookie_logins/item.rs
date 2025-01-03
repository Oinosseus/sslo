use std::sync::Weak;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use sqlx::SqlitePool;
use super::row::CookieLoginDbRow;
use super::super::super::members::{MembersDbData, MembersDbInterface};
use super::super::users::UserInterface;
use super::tablename;

/// The actual data of an item that is shared by Arc<RwLock<ItemData>>
pub(super) struct CookieLoginData {
    pool: SqlitePool,
    pub(super) row: CookieLoginDbRow,
    db_members: Weak<RwLock<MembersDbData>>,

    /// only available after new item is created, unset after calling get_cookie()
    pub(super) decrypted_token: Option<String>,
}

impl CookieLoginData {
    pub fn new(pool: &SqlitePool, row: CookieLoginDbRow, db_members: Weak<RwLock<MembersDbData>>) -> Arc<RwLock<CookieLoginData>> {
        Arc::new(RwLock::new(Self {
            pool: pool.clone(),
            row,
            db_members,
            decrypted_token: None,
        }))
    }
}

/// This abstracts data access to shared items
pub struct CookieLoginInterface(Arc<RwLock<CookieLoginData>>);

impl CookieLoginInterface {
    /// Set up an object from shared data (assumed to be retrieved from database)
    pub(super) fn new(item_data: Arc<RwLock<CookieLoginData>>) -> Self {
        Self(item_data)
    }

    pub async fn id(&self) -> i64 { self.0.read().await.row.rowid }

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

    /// returns the cookie which can be directly send as http header
    /// This only works once, directly after creation of the CookieLogin item
    pub async fn get_cookie(&self) -> Option<String> {
        let mut data = self.0.write().await;
        match data.decrypted_token.take() {
            None => {
                log::warn!("cannot retrieve decrypted token for rowid={}, user={}", data.row.rowid, data.row.user);
                return None;
            },
            Some(decrypted_token) => {
                let cookie = format!("cookie_login={}:{}; HttpOnly; Max-Age=31536000; SameSite=Strict; Partitioned; Secure; Path=/;",
                                     data.row.rowid, decrypted_token);
                return Some(cookie);
            },
        }
    }

    /// verify that a token from within a cookie is valid (updates last usage)
    pub async fn verify(&self, token_decrypted: String, user_agent: String) -> bool {
        let mut data = self.0.write().await;

        // verify token
        let token = sslo_lib::token::Token::new(token_decrypted, data.row.token.clone());
        if !token.verify() { return false; };

        // update usage
        data.row.last_usage = Some(Utc::now());
        data.row.last_useragent = Some(user_agent);
        let pool = data.pool.clone();
        return match data.row.store(&pool).await {
            Ok(_) => true,
            Err(e) => {
                log::error!("failed to update usage for CookieLogin rowid={}, user={}: {}", data.row.rowid, data.row.user, e);
                false
            }
        }
    }

    pub async fn last_useragent(&self) -> Option<String> { self.0.read().await.row.last_useragent.clone() }
    pub async fn last_usage(&self) -> Option<DateTime<Utc>> { self.0.read().await.row.last_usage.clone() }

    /// returns a http header to unset cookie
    pub(super) async fn delete(self) -> String {
        let mut data = self.0.write().await;
        let id = data.row.rowid;
        let pool = data.pool.clone();
        if let Err(e) = data.row.delete(&pool).await {
            log::error!("failed to delete cookie rowid={}: {}", id, e);
        }
        "cookie_login=\"\"; HttpOnly; Max-Age=-1; SameSite=Strict; Partitioned; Secure; Path=/;".to_string()
    }
}
