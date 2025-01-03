macro_rules! tablename {
    () => { "cookie_logins" };
}

use std::collections::HashMap;
use std::sync::{Arc, Weak};
use regex::Regex;
use chrono::{DateTime, Utc};
use sqlx::{Sqlite, SqlitePool};
use tokio::sync::RwLock;
use sslo_lib::error::SsloError;
use sslo_lib::token::{Token, TokenType};
use super::{MembersDbData, MembersDbInterface};
use super::users::UserItem;

/// Data structure that is used for database interaction (only module internal use)
#[derive(sqlx::FromRow, Clone)]
struct DbDataRow {
    rowid: i64,
    user: i64,
    token: String,
    creation: DateTime<Utc>,
    last_useragent: Option<String>,
    last_usage: Option<DateTime<Utc>>,
}

impl DbDataRow {

    fn new(rowid: i64) -> Self {
        debug_assert!(rowid >= 0);
        Self {
            rowid,
            user: 0,
            token: String::new(),
            creation: Utc::now(),
            last_usage: None,
            last_useragent: None,
        }
    }

    async fn load(self: &mut Self, pool: &SqlitePool) -> Result<(), SsloError> {
        return match sqlx::query_as::<Sqlite, DbDataRow>(concat!("SELECT rowid,* FROM ", tablename!(), " WHERE rowid = $1 LIMIT 2;"))
            .bind(self.rowid)
            .fetch_one(pool)
            .await {
            Ok(row) => {
                row.clone_into(self);
                Ok(())
            },
            Err(sqlx::Error::RowNotFound) => {
                Err(SsloError::DatabaseIdNotFound(tablename!(), "rowid", self.rowid))
            },
            Err(e) => {
                Err(SsloError::DatabaseSqlx(e))
            }
        };
    }

    async fn from_user_latest_usage(pool: &SqlitePool, user_id: i64) -> Result<Self, SsloError> {
        return match sqlx::query_as::<Sqlite, Self>(concat!("SELECT rowid,* FROM ", tablename!(), " WHERE user = $1 ORDER BY last_usage DESC LIMIT 1;"))
            .bind(user_id)
            .fetch_one(pool)
            .await {
            Ok(row) => {
                Ok(row)
            },
            Err(sqlx::Error::RowNotFound) => {
                Err(SsloError::DatabaseIdNotFound(tablename!(), "user", user_id))
            },
            Err(e) => {
                Err(SsloError::DatabaseSqlx(e))
            }
        };
    }

    async fn store(self: &mut Self, pool: &SqlitePool) -> Result<(), SsloError> {

        // define query
        let mut query = match self.rowid {
            0 => {
                sqlx::query(concat!("INSERT INTO ", tablename!(),
                "(user,\
                  token,\
                  creation,\
                  last_usage,\
                  last_useragent) \
                  VALUES ($1, $2, $3, $4, $5) RETURNING rowid;"))
            },
            _ => {
                sqlx::query(concat!("UPDATE ", tablename!(), " SET \
                                   user=$1,\
                                   token=$2,\
                                   creation=$3,\
                                   last_usage=$4,\
                                   last_useragent=$5 \
                                   WHERE rowid=$6;"))
            }
        };

        // bind values
        query = query.bind(&self.user)
            .bind(&self.token)
            .bind(&self.creation)
            .bind(&self.last_usage)
            .bind(&self.last_useragent);
        if self.rowid != 0 {
            query = query.bind(self.rowid);
        }

        // execute query
        let res = query.execute(pool).await?;
        if self.rowid == 0 {
            self.rowid = res.last_insert_rowid();
        }
        return Ok(())
    }

    async fn delete(self: &mut Self, pool: &SqlitePool) -> Result<(), SsloError> {
        return match sqlx::query(concat!("DELETE FROM ", tablename!(), " WHERE rowid = $1;"))
            .bind(self.rowid)
            .execute(pool)
            .await {
            Ok(_) => {
                self.rowid = 0;
                self.token = "".to_string();
                self.user = 0;
                Ok(())
            },
            Err(sqlx::Error::RowNotFound) => {
                Err(SsloError::DatabaseIdNotFound(tablename!(), "rowid", self.rowid))
            },
            Err(e) => {
                Err(SsloError::DatabaseSqlx(e))
            }
        };

    }
}

/// The actual data of an item that is shared by Arc<RwLock<ItemData>>
pub(super) struct CookieLoginItemData {
    pool: SqlitePool,
    pub(super) row: DbDataRow,
    db_members: Weak<RwLock<MembersDbData>>,

    /// only available after new item is created, unset after calling get_cookie()
    pub(super) decrypted_token: Option<String>,
}

impl CookieLoginItemData {
    pub fn new(pool: &SqlitePool, row: DbDataRow, db_members: Weak<RwLock<MembersDbData>>) -> Arc<RwLock<CookieLoginItemData>> {
        Arc::new(RwLock::new(Self {
            pool: pool.clone(),
            row,
            db_members,
            decrypted_token: None,
        }))
    }
}

/// This abstracts data access to shared items
pub struct CookieLoginItem(Arc<RwLock<CookieLoginItemData>>);

impl CookieLoginItem {
    /// Set up an object from shared data (assumed to be retrieved from database)
    fn new(item_data: Arc<RwLock<CookieLoginItemData>>) -> Self {
        Self(item_data)
    }

    pub async fn id(&self) -> i64 { self.0.read().await.row.rowid }

    pub async fn user(&self) -> Option<UserItem> {
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
    async fn delete(self) -> String {
        let mut data = self.0.write().await;
        let id = data.row.rowid;
        let pool = data.pool.clone();
        if let Err(e) = data.row.delete(&pool).await {
            log::error!("failed to delete cookie rowid={}: {}", id, e);
        }
        "cookie_login=\"\"; HttpOnly; Max-Age=-1; SameSite=Strict; Partitioned; Secure; Path=/;".to_string()
    }
}

pub(super) struct CookieLoginTableData {
    pool: SqlitePool,
    item_cache: HashMap<i64, Arc<RwLock<CookieLoginItemData>>>,
    db_members: Weak<RwLock<MembersDbData>>,
}

impl CookieLoginTableData {
    pub(super) fn new(pool: SqlitePool, db_members: Weak<RwLock<MembersDbData>>) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            pool,
            item_cache: HashMap::new(),
            db_members,
        }))
    }
}

pub struct CookieLoginTable(Arc<RwLock<CookieLoginTableData>>);

impl CookieLoginTable {
    pub(super) fn new(data: Arc<RwLock<CookieLoginTableData>>) -> Self { Self(data) }

    /// Get an item
    /// First tries loading from cache, then from database
    pub async fn item_by_id(&self, id: i64) -> Option<CookieLoginItem> {

        {   // try cache hit
            let tbl_data = self.0.read().await;
            if let Some(item_data) = tbl_data.item_cache.get(&id) {
                return Some(CookieLoginItem::new(item_data.clone()));
            }
        }

        {   // try loading from DB if not found in cache
            let mut tbl_data = self.0.write().await;

            // load from db
            let mut row = DbDataRow::new(id);
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
            let item_data = CookieLoginItemData::new(&tbl_data.pool, row, tbl_data.db_members.clone());
            let item = CookieLoginItem::new(item_data.clone());
            tbl_data.item_cache.insert(id, item_data);
            return Some(item);
        }
    }

    /// Parsing a cookie header and return an item
    /// This verifies the token and updates usage info
    pub async fn item_by_cookie(&self, useragent: String, cookie: &str) -> Option<CookieLoginItem> {

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
        let item = match self.item_by_id(cookie_id).await {
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

    pub async fn create_new_cookie(&self, user: &UserItem) -> Option<CookieLoginItem> {
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
        let mut row = DbDataRow::new(0);
        row.user = user_id;
        row.creation = Utc::now();
        row.token = token.encrypted;
        if let Err(e) = row.store(&tbl_data.pool.clone()).await {
            log::error!("failed store new cookie for user id={}: {}", user_id, e);
            return None;
        }
        let new_row_id = row.rowid;

        // create item
        let item_data = CookieLoginItemData::new(&tbl_data.pool.clone(), row, tbl_data.db_members.clone());
        {
            let mut item_data_mut = item_data.write().await;
            item_data_mut.decrypted_token = Some(token.decrypted);
        }

        // update cache
        tbl_data.item_cache.insert(new_row_id, item_data.clone());

        // return interface
        Some(CookieLoginItem::new(item_data))
    }

    pub async fn item_from_latest_usage(&self, user: &UserItem) -> Option<CookieLoginItem> {
        let mut row : Option<DbDataRow> = None;

        {   // find item in db, with local lock-scope
            let data = self.0.read().await;
            let pool = data.pool.clone();
            if let Ok(r) = DbDataRow::from_user_latest_usage(&pool, user.id().await).await {
                row = Some(r);
            }
        }

        return match row {
            None => {None}
            Some(row) => {
                self.item_by_id(row.rowid).await
            }
        }
    }

    /// returns a http header to unset cookie
    pub async fn delete_cookie(&self, cookie_login: CookieLoginItem) -> String {
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

    mod row {
        use super::*;
        use test_log::test;

        #[test(tokio::test)]
        async fn new_defaults() {
            let row = DbDataRow::new(33);
            assert_eq!(row.rowid, 33);
            assert_eq!(row.user, 0);
            assert_eq!(row.token, String::new());
            assert_eq!(row.last_usage, None);
            assert_eq!(row.last_useragent, None);
        }

        /// Testing load and store (insert+update)
        #[test(tokio::test)]
        async fn load_store_delete() {
            let pool = get_pool().await;

            // define some UTC times
            let dt1: DateTime<Utc> = DateTime::parse_from_rfc3339("1001-01-01T01:01:01.1111+01:00").unwrap().into();
            let dt2: DateTime<Utc> = DateTime::parse_from_rfc3339("2002-02-02T02:02:02.2222+02:00").unwrap().into();
            let dt3: DateTime<Utc> = DateTime::parse_from_rfc3339("3003-03-03T03:03:03.3333+03:00").unwrap().into();

            // store (insert)
            let mut row = DbDataRow::new(0);
            row.user = 44;
            row.token = "MyInsecureTestToken".to_string();
            row.creation = dt1;
            row.last_usage = Some(dt2);
            row.last_useragent = Some("unit test".to_string());
            row.store(&pool).await.unwrap();

            // load
            let mut row = DbDataRow::new(1);
            row.load(&pool).await.unwrap();
            assert_eq!(row.rowid, 1);
            assert_eq!(row.user, 44);
            assert_eq!(row.token, "MyInsecureTestToken".to_string());
            assert_eq!(row.creation, dt1.clone());
            assert_eq!(row.last_usage, Some(dt2.clone()));
            assert_eq!(row.last_useragent, Some("unit test".to_string()));

            // store (update)
            let mut row = DbDataRow::new(1);
            row.user = 46;
            row.token = "MyNewInsecureTestToken".to_string();
            row.creation = dt2;
            row.last_usage = Some(dt3);
            row.last_useragent = Some("new unit test".to_string());
            row.store(&pool).await.unwrap();

            // load
            let mut row = DbDataRow::new(1);
            row.load(&pool).await.unwrap();
            assert_eq!(row.rowid, 1);
            assert_eq!(row.user, 46);
            assert_eq!(row.token, "MyNewInsecureTestToken".to_string());
            assert_eq!(row.creation, dt2.clone());
            assert_eq!(row.last_usage, Some(dt3.clone()));
            assert_eq!(row.last_useragent, Some("new unit test".to_string()));

            // delete
            row.delete(&pool).await.unwrap();
            assert_eq!(row.rowid, 0);
            assert_eq!(row.user, 0);
            assert_eq!(row.token, "".to_string());
        }
    }
}
