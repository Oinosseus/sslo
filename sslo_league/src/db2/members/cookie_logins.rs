mod row;
mod item;
pub use item::CookieLoginInterface;

/// This is the central defined name of the table in this module,
/// used to allow copy&paste of the code for other tables.
macro_rules! tablename {
    () => { "cookie_logins" };
}

use std::collections::HashMap;
use std::sync::{Arc, Weak};
use chrono::Utc;
use regex::Regex;
use sqlx::SqlitePool;
use tokio::sync::RwLock;
use sslo_lib::error::SsloError;
use sslo_lib::token::{Token, TokenType};
pub(self) use tablename;
use item::CookieLoginData;
use row::CookieLoginDbRow;
use super::MembersDbData;
use super::users::UserInterface;

pub(super) struct CookieLoginTableData {
    pool: SqlitePool,
    item_cache: HashMap<i64, Arc<RwLock<item::CookieLoginData>>>,
    db_members: Weak<RwLock<MembersDbData>>,
}

impl CookieLoginTableData {
    pub fn new(pool: SqlitePool, db_members: Weak<RwLock<MembersDbData>>) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            pool,
            item_cache: HashMap::new(),
            db_members,
        }))
    }
}

pub struct CookieLoginTableInterface(Arc<RwLock<CookieLoginTableData>>);

impl CookieLoginTableInterface {
    pub fn new(data: Arc<RwLock<CookieLoginTableData>>) -> Self { Self(data) }

    /// Get an item
    /// This first tries to load the item from cache,
    /// and secondly load it from the database.
    pub async fn from_id(&self, id: i64) -> Option<CookieLoginInterface> {

        {   // try cache hit
            let tbl_data = self.0.read().await;
            if let Some(item_data) = tbl_data.item_cache.get(&id) {
                return Some(CookieLoginInterface::new(item_data.clone()));
            }
        }

        {   // try loading from DB if not found in cache
            let mut tbl_data = self.0.write().await;

            // load from db
            let mut row = CookieLoginDbRow::new(id);
            match row.load(&tbl_data.pool).await {
                Ok(_) => { },
                Err(e) => {
                    if e.is_db_not_found_type() {
                        log::warn!("{}", e);
                    } else {
                        log::error!("{}", e.to_string());
                    }
                    return None;
                },
            }
            debug_assert_eq!(row.rowid, id);

            // create item
            let item_data = CookieLoginData::new(&tbl_data.pool, row, tbl_data.db_members.clone());
            let item = CookieLoginInterface::new(item_data.clone());
            tbl_data.item_cache.insert(id, item_data);
            return Some(item);
        }
    }

    /// Parsing a cookie header and return an item
    /// This verifies the token and updates usage info
    pub async fn from_cookie(&self, useragent: String, cookie: &str) -> Option<CookieLoginInterface> {

        // quick check
        if cookie.find("cookie_login=").is_none() {
            return None;
        };

        // chop cookie string
        let re = Regex::new(r"^cookie_login=([0-9]+):([a-f0-9]+).*$").unwrap();
        let groups = match re.captures(cookie) {
            Some(x) => x,
            None => {
                log::warn!("Invalid cookie format: '{}'", cookie);
                return None;
            }
        };
        let cookie_id: i64 = match groups.get(1)?.as_str().parse::<i64>() {
            Ok(id) => id,
            Err(_) => return None,
        };
        let cookie_token_decrypted: String = groups.get(2)?.as_str().into();

        // find id in database
        let item = match self.from_id(cookie_id).await {
            None => {
                log::warn!("no CookieLogin for rowid={} found", cookie_id);
                return None;
            }
            Some(item) => {item}
        };

        // verify token
        if !item.verify(cookie_token_decrypted, useragent).await {
            log::warn!("failed to verify cookie for Cookie id {}", cookie_id);
            return None;
        }

        Some(item)
    }

    pub async fn create_new_cookie(&self, user: &UserInterface) -> Option<CookieLoginInterface> {
        let mut tbl_data = self.0.write().await;
        let user_id = user.id().await;

        // create a new token
        let token = match Token::generate(TokenType::Quick) {
            Ok(token) => token,
            Err(e) => {
                log::error!("Could not generate new token: {}", e);
                return None;
            }
        };

        // create a new row
        let mut row = CookieLoginDbRow::new(0);
        row.user = user_id;
        row.creation = Utc::now();
        row.token = token.encrypted;
        if let Err(e) = row.store(&tbl_data.pool.clone()).await {
            log::error!("failed store new cookie for user id={}: {}", user_id, e);
            return None;
        }
        let new_row_id = row.rowid;

        // create item
        let item_data = CookieLoginData::new(&tbl_data.pool.clone(), row, tbl_data.db_members.clone());
        {
            let mut item_data_mut = item_data.write().await;
            item_data_mut.decrypted_token = Some(token.decrypted);
        }

        // update cache
        tbl_data.item_cache.insert(new_row_id, item_data.clone());

        // return interface
        Some(CookieLoginInterface::new(item_data))
    }

    pub async fn item_from_latest_usage(&self, user: &UserInterface) -> Option<CookieLoginInterface> {
        let mut row : Option<CookieLoginDbRow> = None;

        {   // find item in db, with local lock-scope
            let data = self.0.read().await;
            let pool = data.pool.clone();
            if let Ok(r) = CookieLoginDbRow::from_user_latest_usage(&pool, user.id().await).await {
                row = Some(r);
            }
        }

        return match row {
            None => {None}
            Some(row) => {
                self.from_id(row.rowid).await
            }
        }
    }

    /// returns a http header to unset cookie
    pub async fn delete_cookie(&self, cookie_login: CookieLoginInterface) -> String {
        let id = cookie_login.id().await;

        {   // remove from cache
            let mut data = self.0.write().await;
            data.item_cache.remove(&id);
        }

        // delete item
        if let Some(user) = cookie_login.user().await {
            log::info!("logout user {}, from cookie {}", user.id().await, cookie_login.id().await);
        } else {
            log::warn!("cookie deletion without associated user");
            log::info!("logout from cookie {}", cookie_login.id().await);
        }
        cookie_login.delete().await
    }
}
