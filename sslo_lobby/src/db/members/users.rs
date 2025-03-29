macro_rules! tablename {
    () => { "users" };
}

use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::Sub;
use tokio::sync::RwLock;
use std::sync::{Arc, Weak};
use chrono::{DateTime, Utc};
use sqlx::{Sqlite, SqlitePool};
use rand::RngCore;
use sslo_lib::error::SsloError;
use sslo_lib::optional_date::OptionalDateTime;
use sslo_lib::token::{Token, TokenType};
use super::MembersDbData;

#[derive(sqlx::FromRow, Clone)]
struct DbDataRow {
    pub(super) rowid: i64,
    pub(super) name: String,
}

impl DbDataRow {

    /// Create a new (empty/default) data row
    fn new(rowid: i64) -> Self {
        debug_assert!(rowid >= 0);
        Self {
            rowid,
            name: "".to_string(),
        }
    }

    /// Read the data from the database
    /// This consumes a Row object and returns a new row object on success
    async fn load(self: &mut Self, pool: &SqlitePool) -> Result<(), SsloError> {
        match sqlx::query_as::<Sqlite, DbDataRow>(concat!("SELECT rowid,* FROM ", tablename!(), " WHERE rowid = $1 LIMIT 2;"))
            .bind(self.rowid)
            .fetch_one(pool)
            .await {
            Ok(row) => {
                row.clone_into(self);
                return Ok(());
            },
            Err(sqlx::Error::RowNotFound) => {
                return Err(SsloError::DatabaseIdNotFound(tablename!(), "rowid", self.rowid));
            },
            Err(e) => {
                return Err(SsloError::DatabaseSqlx(e));
            }
        };
    }

    /// Write the data into the database
    /// When rowid is unequal to '0', an UPDATE is executed,
    /// When rowid is zero, an insert is executed and rowid is updated
    /// When INSERT fails, rowid will stay at zero
    async fn store(self: &mut Self, pool: &SqlitePool) -> Result<(), SsloError> {

        // define query
        let mut query = match self.rowid {
            0 => {
                sqlx::query(concat!("INSERT INTO ", tablename!(),
                "(name) VALUES ($1) RETURNING rowid;"))
            },
            _ => {
                sqlx::query(concat!("UPDATE ", tablename!(), " SET name=$1 WHERE rowid=$2;"))
            }
        };

        // bind values
        query = query.bind(&self.name);
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

    /// Returns a string that can be used for integrating this row into a log message
    fn display(&self) -> String {
        format!("{}(id={};name={})", tablename!(), self.rowid, self.name)
    }
}


/// The actual data of an item that is shared by Arc<RwLock<ItemData>>
struct UserItemData {
    pool: Option<SqlitePool>,  // dummy users do not have a pool
    row: DbDataRow,
    db_members: Weak<RwLock<MembersDbData>>,
}

impl UserItemData {
    fn new(pool: &SqlitePool, row: DbDataRow, db_members: Weak<RwLock<MembersDbData>>) -> Arc<RwLock<UserItemData>> {
        Arc::new(RwLock::new(Self {
            pool: Some(pool.clone()),
            row,
            db_members,
        }))
    }

    ///! Creates a new user, which is actually not stored in the database
    fn new_dummy() -> Arc<RwLock<UserItemData>> {
        let row = DbDataRow::new(0);
        Arc::new(RwLock::new(Self {
            pool: None,
            row,
            db_members: Weak::new(),
        }))
    }
}

/// This abstracts data access to shared database items
#[derive(Clone)]
pub struct UserItem(Arc<RwLock<UserItemData>>);

impl UserItem {

    /// Set up an object from shared data (assumed to be retrieved from database)
    fn new(item_data: Arc<RwLock<UserItemData>>) -> Self {
        Self(item_data)
    }

    /// Returns a string, that can be used in log messages
    pub async fn display(&self) -> String {
        self.0.read().await.row.display()
    }

    /// The database rowid
    pub async fn id(&self) -> i64 {
        self.0.read().await.row.rowid
    }

    /// the user's name
    pub async fn name(&self) -> String {
        self.0.read().await.row.name.clone()
    }

    /// the user's name for html presentation
    pub async fn html_name(&self) -> String {
        let mut html = String::new();
        html_escape::encode_safe_to_string(&self.0.read().await.row.name, &mut html);
        return html;
    }

    /// Update the user's name
    pub async fn set_name(self: &mut Self, name: String) -> Result<(), SsloError> {
        let mut data = self.0.write().await;
        let name = name.trim().to_string();
        let display_before = data.row.display();
        data.row.name = name;
        let display_after = data.row.display();
        log::info!("Change name from {} to {}", display_before, display_after);
        match data.pool.clone() {
            None => Ok(()),
            Some(pool) => data.row.store(&pool).await
        }
    }
}


pub(super) struct UserTableData {
    pool: SqlitePool,
    item_cache: HashMap<i64, Arc<RwLock<UserItemData>>>
}

impl UserTableData {
    pub(super) fn new(pool: SqlitePool) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            pool,
            item_cache: HashMap::new(),
        }))
    }
}

pub struct UserTable(
    Arc<RwLock<UserTableData>>
);

impl UserTable {

    pub(super) fn new(data: Arc<RwLock<UserTableData>>) -> Self {
        Self(data)
    }

    /// Create a new user
    pub async fn create_new_user(&self) -> Option<UserItem> {

        // insert new item into DB
        let mut row = DbDataRow::new(0);
        {
            let pool = self.0.read().await.pool.clone();
            match row.store(&pool).await {
                Ok(_) => {}
                Err(e) => {
                    log::error!("Could not create a new {}: {}", row.display(), e);
                    return None;
                }
            }
        }
        let id = row.rowid;

        // update cache
        let mut tbl_data = self.0.write().await;
        let row_display = row.display();
        let item_data = UserItemData::new(&tbl_data.pool, row, Weak::new());
        let item = UserItem::new(item_data.clone());
        tbl_data.item_cache.insert(item.id().await, item_data);

        // log
        log::info!("new user created: {}", row_display);

        // done
        return Some(item);
    }

    /// Get a dummy user
    /// This can be used to handle unknown users (will not be stored into db)
    pub async fn user_dummy(&self) -> UserItem {
        let item_data = UserItemData::new_dummy();
        UserItem::new(item_data)
    }

    /// Get an item
    /// This first tries to load the item from cache,
    /// and secondly load it from the database.
    pub async fn user_by_id(&self, id: i64) -> Option<UserItem> {

        // sanity check
        debug_assert!(id > 0);
        if id <= 0 {
            log::error!("Deny to retrieve user with nagive rowid={}", id);
            return None;
        }

        // try cache hit
        {
            let tbl_data = self.0.read().await;
            if let Some(item_data) = tbl_data.item_cache.get(&id) {
                return Some(UserItem::new(item_data.clone()));
            }
        }

        // try loading from DB if not found in cache
        {
            let mut tbl_data = self.0.write().await;

            // load from db row
            let mut row = DbDataRow::new(id);
            match row.load(&tbl_data.pool).await {
                Ok(_) => { },
                Err(e) => {
                    if e.is_db_not_found_type() {
                        log::warn!("no user found with rowid={}, {}", id, e);
                    } else {
                        log::error!("failed to load {}: {}", row.display(), e.to_string());
                    }
                    return None;
                },
            }
            debug_assert_eq!(row.rowid, id);

            // create item
            let item_data = UserItemData::new(&tbl_data.pool, row, Weak::new());
            let item = UserItem::new(item_data.clone());
            tbl_data.item_cache.insert(id, item_data);
            return Some(item);
        }
    }
}

#[cfg(test)]
mod tests {
    use sqlx::SqlitePool;
    use super::*;
    use test_log::test;

    async fn get_pool() -> SqlitePool {
        let pool = sslo_lib::db::get_pool(None);
        sqlx::migrate!("../rsc/db_migrations/lobby_members").run(&pool).await.unwrap();
        return pool;
    }

    async fn get_table_interface() -> UserTable {
        let pool = get_pool().await;
        let tbl_data = UserTableData::new(pool);
        UserTable::new(tbl_data.clone())
    }

    mod table {
        use test_log::test;

        #[test(tokio::test)]
        async fn create_new_user() {
            let tbl = super::get_table_interface().await;
            assert_eq!(tbl.0.read().await.item_cache.len(), 0);
            let item = tbl.create_new_user().await.unwrap();
            assert_eq!(tbl.0.read().await.item_cache.len(), 1);
            assert_eq!(item.id().await, 1);
        }

        #[test(tokio::test)]
        async fn user_by_id() {
            let tbl = super::get_table_interface().await;

            // check if cache is empty
            {
                let cache = tbl.0.read().await;
                assert_eq!(cache.item_cache.len(), 0);
            }

            // append items to db_obsolete
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
    }

    mod item {
        use chrono::{DateTime, Utc};
        use sqlx::SqlitePool;
        use super::super::*;
        use test_log::test;

        async fn create_new_item(pool: &SqlitePool) -> UserItem {
            let row = DbDataRow::new(0);
            let data = UserItemData::new(pool, row, Weak::new());
            UserItem::new(data)
        }

        async fn load_item_from_db(id: i64, pool: &SqlitePool) -> UserItem {
            let mut row = DbDataRow::new(id);
            row.load(pool).await.unwrap();
            let data = UserItemData::new(&pool, row, Weak::new());
            UserItem::new(data)
        }

        /// test item generation and property access
        #[test(tokio::test)]
        async fn new_item() {
            let pool = super::get_pool().await;

            // create item
            let row = DbDataRow::new(0);
            let data = UserItemData::new(&pool, row, Weak::new());
            let item = UserItem::new(data);
            assert_eq!(item.id().await, 0);
            assert_eq!(item.name().await, "");
        }

        #[test(tokio::test)]
        async fn name_and_id() {
            let pool = super::get_pool().await;

            // create item
            let mut item = create_new_item(&pool.clone()).await;
            assert_eq!(item.id().await, 0);

            // modify item
            assert_eq!(item.name().await, "");
            item.set_name(" Ronald Antonio \"Ronnie\" O'Sullivan\n".to_string()).await.unwrap();
            assert_eq!(item.id().await, 1);
            assert_eq!(item.name().await, "Ronald Antonio \"Ronnie\" O'Sullivan");

            // check if arrived in database
            let item = load_item_from_db(1, &pool).await;
            assert_eq!(item.id().await, 1);
            assert_eq!(item.name().await, "Ronald Antonio \"Ronnie\" O'Sullivan");

            // check html name
            assert_eq!(item.html_name().await, "Ronald Antonio &quot;Ronnie&quot; O&#x27;Sullivan");
        }
    }

    mod row {
        // use chrono::{DateTime, Utc};
        use super::*;
        use test_log::test;

        #[test(tokio::test)]
        async fn new_defaults() {
            let row = DbDataRow::new(33);
            assert_eq!(row.rowid, 33);
            assert_eq!(row.name, "".to_string());
        }

        /// Testing load and store (insert+update)
        #[test(tokio::test)]
        async fn load_store() {
            let pool = super::get_pool().await;

            // define some UTC times
            // let dt1: DateTime<Utc> = DateTime::parse_from_rfc3339("1001-01-01T01:01:01.1111+01:00").unwrap().into();
            // let dt2: DateTime<Utc> = DateTime::parse_from_rfc3339("2002-02-02T02:02:02.2222+02:00").unwrap().into();
            // let dt3: DateTime<Utc> = DateTime::parse_from_rfc3339("3003-03-03T03:03:03.3333+03:00").unwrap().into();
            // let dt4: DateTime<Utc> = DateTime::parse_from_rfc3339("4004-04-04T04:04:04.4444+04:00").unwrap().into();
            // let dt5: DateTime<Utc> = DateTime::parse_from_rfc3339("5005-05-05T05:05:05.5555+05:00").unwrap().into();

            // store (insert)
            let mut row = DbDataRow::new(0);
            row.name = "RowName".to_string();
            row.store(&pool).await.unwrap();

            // load
            let mut row = DbDataRow::new(1);
            row.load(&pool).await.unwrap();
            assert_eq!(row.rowid, 1);
            assert_eq!(row.name, "RowName".to_string());

            // store (update)
            let mut row = DbDataRow::new(1);
            row.name = "RowNameNew".to_string();
            row.store(&pool).await.unwrap();

            // load
            let mut row = DbDataRow::new(1);
            row.load(&pool).await.unwrap();
            assert_eq!(row.rowid, 1);
            assert_eq!(row.name, "RowNameNew".to_string());
        }
    }
}
