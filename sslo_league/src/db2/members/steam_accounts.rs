macro_rules! tablename {
    {} => { "steam_accounts" };
}

use std::collections::HashMap;
use std::sync::{Arc, Weak};
use chrono::{DateTime, Utc};
use sqlx::{Sqlite, SqlitePool};
use tokio::sync::RwLock;
use super::{MembersDbData, MembersDbInterface};
use super::users::UserItem;
use sslo_lib::error::SsloError;
use sslo_lib::optional_date::OptionalDateTime;
use crate::db2::members::email_accounts::EmailAccountItem;

#[derive(sqlx::FromRow, Clone)]
struct DbDataRow {
    rowid: i64,
    user: Option<i64>,
    steam_id: String,
    creation: DateTime<Utc>,
    last_login: Option<DateTime<Utc>>,
}

impl DbDataRow {

    /// Create a new (empty/default) data row
    fn new(rowid: i64) -> Self {
        debug_assert!(rowid >= 0);
        Self {
            rowid,
            user: None,
            steam_id: String::new(),
            creation: Utc::now(),
            last_login: None,
        }
    }

    /// directly retrieve an item from database by steam_id
    async fn from_steam_id(steam_id: &str, pool: &SqlitePool) -> Result<Self, SsloError> {
        return match sqlx::query_as::<Sqlite, DbDataRow>(concat!("SELECT rowid,* FROM ", tablename!(), " WHERE steam_id = $1 LIMIT 2;"))
            .bind(steam_id)
            .fetch_one(pool)
            .await {
            Ok(row) => {
                Ok(row)
            },
            Err(sqlx::Error::RowNotFound) => {
                Err(SsloError::DatabaseDataNotFound(tablename!(), "steam_id", steam_id.to_string()))
            },
            Err(e) => {
                return Err(SsloError::DatabaseSqlx(e));
            }
        };
    }

    async fn from_user(user: &UserItem, pool: &SqlitePool) -> Vec<Self> {
        let user_id = user.id().await;
        match sqlx::query_as::<Sqlite, DbDataRow>(concat!("SELECT rowid,* FROM ", tablename!(), " WHERE user = $1 LIMIT 100;"))
            .bind(user_id)
            .fetch_all(pool)
            .await {
            Ok(rows) => {
                if rows.len() >= 99 {
                    log::warn!("user rowid={} has more than 99 email accounts associated (truncating for safety)", user_id);
                }
                rows
            },
            Err(e) => {
                log::error!("{}", e);
                Vec::new()
            }
        }
    }

    /// Read the data from the database
    /// This consumes a Row object and returns a new row object on success
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

    /// Write the data into the database
    /// When rowid is unequal to '0', an UPDATE is executed,
    /// When rowid is zero, an insert is executed and rowid is updated
    /// When INSERT fails, rowid will stay at zero
    async fn store(self: &mut Self, pool: &SqlitePool) -> Result<(), SsloError> {

        // define query
        let mut query = match self.rowid {
            0 => {
                sqlx::query(concat!("INSERT INTO ", tablename!(),
                "(user,\
                  steam_id,\
                  creation,\
                  last_login) \
                  VALUES ($1, $2, $3, $4) RETURNING rowid;"))
            },
            _ => {
                sqlx::query(concat!("UPDATE ", tablename!(), " SET \
                                   user=$1,\
                                   steam_id=$2,\
                                   creation=$3,\
                                   last_login=$4 \
                                   WHERE rowid=$5;"))
            }
        };

        // bind values
        query = query.bind(&self.user)
            .bind(&self.steam_id)
            .bind(&self.creation)
            .bind(&self.last_login);
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
        if let Some(user_id) = self.user {
            format!("{}(id={};user-id={};steam-id={})", tablename!(), self.rowid, user_id, self.steam_id)
        } else {
            format!("{}(id={};user-id=None;steam-id={})", tablename!(), self.rowid, self.steam_id)
        }
    }
}

pub(super) struct SteamAccountData {
    pool: SqlitePool,
    row: DbDataRow,
    db_members: Weak<RwLock<MembersDbData>>,
}

impl SteamAccountData {
    pub fn new(pool: &SqlitePool, row: DbDataRow, db_members: Weak<RwLock<MembersDbData>>) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            pool: pool.clone(),
            row,
            db_members,
        }))
    }
}

/// This abstracts data access to shared items
pub struct SteamAccountItem(Arc<RwLock<SteamAccountData>>);

impl SteamAccountItem {

    fn new(item_data: Arc<RwLock<SteamAccountData>>) -> Self {
        Self(item_data)
    }

    pub async fn id(&self) -> i64 { self.0.read().await.row.rowid }
    pub async fn display(&self) -> String { self.0.read().await.row.display() }
    pub async fn steam_id(&self) -> String { self.0.read().await.row.steam_id.clone() }
    pub async fn creation(&self) -> DateTime<Utc> { self.0.read().await.row.creation.clone() }

    /// returns the assigned user
    /// If no user is assigned, a new user will be tried to create
    pub async fn user(&self) -> Option<UserItem> {
        {   // try reading existing user
            let data = self.0.read().await;
            if let Some(user_id) = data.row.user {
                let db_members = match data.db_members.upgrade() {
                    Some(db_data) => MembersDbInterface::new(db_data),
                    None => {
                        log::error!("cannot upgrade weak pointer for {}", data.row.display());
                        return None;
                    }
                };
                let tbl_usr = db_members.tbl_users().await;
                return tbl_usr.user_by_id(user_id).await;
            }
        }

        // create new user
        let mut data = self.0.write().await;
        let pool = data.pool.clone();
        let db_members = match data.db_members.upgrade() {
            Some(db_data) => MembersDbInterface::new(db_data),
            None => {
                log::error!("cannot upgrade weak pointer for {}", data.row.display());
                return None;
            }
        };
        let tbl_usr = db_members.tbl_users().await;
        let user = match tbl_usr.create_new_user().await {
            Some(user) => user,
            None => {
                log::error!("failed to create new user for {}", data.row.display());
                return None;
            }
        };
        data.row.user = Some(user.id().await);
        match data.row.store(&pool).await {
            Ok(_) => {},
            Err(e) => {
                log::error!("failed to store new user for {}", data.row.display());
                return None;
            }
        }
        return Some(user);
    }

    /// Returns true, if a user is assigned to this steam account
    pub async fn has_user(&self) -> bool {
        let data = self.0.read().await;
        data.row.user.is_some()
    }

    /// Assign a new user
    /// Checking if user is already set, before writing to disk
    pub async fn set_user(&self, user: &UserItem) -> Result<(), SsloError> {
        let mut data = self.0.write().await;

        // check if user is already set
        if let Some(&existing_user) = data.row.user.as_ref() {
            if existing_user == user.id().await {
                return Ok(());
            }
        }

        // save
        data.row.user = Some(user.id().await);
        let pool = data.pool.clone();
        data.row.store(&pool).await
    }

    pub async fn last_login(&self) -> OptionalDateTime {
        let data = self.0.read().await;
        OptionalDateTime::new(data.row.last_login.clone())
    }

    pub async fn set_last_login(&self, last_login: DateTime<Utc>) -> Result<(), SsloError> {
        let mut data = self.0.write().await;
        data.row.last_login = Some(last_login);
        let pool = data.pool.clone();
        data.row.store(&pool).await
    }

}

pub(super) struct SteamAccountsTableData {
    pool: SqlitePool,
    item_cache_by_rowid: HashMap<i64, Arc<RwLock<SteamAccountData>>>,
    item_cache_by_steamid: HashMap<String, Arc<RwLock<SteamAccountData>>>,
    db_members: Weak<RwLock<MembersDbData>>,
}

impl SteamAccountsTableData {
    pub(super) fn new(pool: SqlitePool, db_members: Weak<RwLock<MembersDbData>>) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            pool,
            item_cache_by_rowid: HashMap::new(),
            item_cache_by_steamid: HashMap::new(),
            db_members,
        }))
    }
}

pub struct SteamAccountsTable(Arc<RwLock<SteamAccountsTableData>>);

impl SteamAccountsTable {
    pub(super) fn new(data: Arc<RwLock<SteamAccountsTableData>>) -> Self { Self(data) }

    /// Get all steam accounts that are associated to a certain user
    pub async fn items_by_user(&self, user: &UserItem) -> Vec<SteamAccountItem> {
        let mut tbl_data = self.0.write().await;
        let pool = tbl_data.pool.clone();
        let mut item_list: Vec<SteamAccountItem> = Vec::new();

        for row in DbDataRow::from_user(user, &pool).await.into_iter() {
            let rowid = row.rowid;
            let item = match tbl_data.item_cache_by_rowid.get(&rowid) {
                Some(item_data) => SteamAccountItem::new(item_data.clone()),
                None => {
                    let item_data = SteamAccountData::new(&pool, row, tbl_data.db_members.clone());
                    let item = SteamAccountItem::new(item_data.clone());
                    tbl_data.item_cache_by_rowid.insert(rowid, item_data.clone());
                    item
                }
            };
            item_list.push(item);
        }

        return item_list;
    }

    /// Get an item
    /// This first tries to load the item from cache,
    /// and secondly load it from the database.
    pub async fn item_by_steam_id(&self, steam_id: &str,
                                  allow_new_item_creation: bool) -> Option<SteamAccountItem> {

        // try cache hit
        {
            let tbl_data = self.0.read().await;
            if let Some(item_data) = tbl_data.item_cache_by_steamid.get(steam_id) {
                return Some(SteamAccountItem::new(item_data.clone()));
            }
        }

        // try loading from DB if not found in cache
        {
            let mut tbl_data = self.0.write().await;

            // load from db table
            let mut row = match DbDataRow::from_steam_id(steam_id, &tbl_data.pool).await {
                Ok(row) => row,
                Err(e) if !e.is_db_not_found_type() => {
                    log::error!("{}", e.to_string());
                    return None;
                },
                Err(e) if allow_new_item_creation => {
                    let mut row = DbDataRow::new(0);
                    row.steam_id = steam_id.to_string();
                    match row.store(&tbl_data.pool).await {
                        Ok(_) => {},
                        Err(e) => {
                            log::error!("{}", e.to_string());
                            return None;
                        }
                    }
                    log::info!("New item created: {}", row.display());
                    row
                },
                Err(_) => {
                    return None;
                },
            };
            debug_assert_eq!(row.steam_id, steam_id);

            // create item
            let item_data = SteamAccountData::new(&tbl_data.pool, row, tbl_data.db_members.clone());
            let item = SteamAccountItem::new(item_data.clone());
            tbl_data.item_cache_by_rowid.insert(item.id().await, item_data.clone());
            tbl_data.item_cache_by_steamid.insert(item.steam_id().await, item_data);
            return Some(item);
        }
    }

}

#[cfg(test)]
mod tests {
    use sqlx::SqlitePool;
    use super::*;
    use test_log::test;
    use crate::db2::members::users::{UserTable, UserTableData};

    async fn get_pool() -> SqlitePool {
        let pool = sslo_lib::db::get_pool(None);
        sqlx::migrate!("../rsc/db_migrations/league_members").run(&pool).await.unwrap();
        pool
    }

    async fn create_new_item(pool: &SqlitePool) -> SteamAccountItem {
        let row = DbDataRow::new(0);
        let data = SteamAccountData::new(pool, row, Weak::new());
        SteamAccountItem::new(data)
    }

    async fn get_table_interface() -> SteamAccountsTable {
        let pool = get_pool().await;
        let tbl_data = SteamAccountsTableData::new(pool, Weak::new());
        SteamAccountsTable::new(tbl_data.clone())
    }

    mod row {
        use test_log::test;
        use super::*;

        #[test(tokio::test)]
        async fn new_defaults() {
            let row = DbDataRow::new(33);
            assert_eq!(row.rowid, 33);
            assert_eq!(row.user, None);
            assert_eq!(row.steam_id, String::new());
            assert_eq!(row.last_login, None);
        }

        /// Testing load and store (insert+update)
        #[test(tokio::test)]
        async fn load_store() {
            let pool = get_pool().await;

            // genertae some test data
            let mut query = sqlx::query("INSERT INTO users (rowid,name) VALUES (44,'Foo');");
            query.execute(&pool).await.unwrap();
            let mut query = sqlx::query("INSERT INTO users (rowid,name) VALUES (46,'Foo');");
            query.execute(&pool).await.unwrap();

            // define some UTC times
            let dt1: DateTime<Utc> = DateTime::parse_from_rfc3339("1001-01-01T01:01:01.1111+01:00").unwrap().into();
            let dt2: DateTime<Utc> = DateTime::parse_from_rfc3339("2002-02-02T02:02:02.2222+02:00").unwrap().into();
            let dt3: DateTime<Utc> = DateTime::parse_from_rfc3339("3003-03-03T03:03:03.3333+03:00").unwrap().into();

            // store (insert)
            let mut row = DbDataRow::new(0);
            row.user = Some(44);
            row.steam_id = "SomeSteam64GUID".to_string();
            row.creation = dt1;
            row.last_login = Some(dt2);
            row.store(&pool).await.unwrap();

            // load
            let mut row = DbDataRow::new(1);
            row.load(&pool).await.unwrap();
            assert_eq!(row.rowid, 1);
            assert_eq!(row.user, Some(44));
            assert_eq!(row.steam_id, "SomeSteam64GUID".to_string());
            assert_eq!(row.creation, dt1.clone());
            assert_eq!(row.last_login, Some(dt2.clone()));

            // store (update)
            let mut row = DbDataRow::new(1);
            row.user = Some(46);
            row.steam_id = "NewSomeSteam64GUID".to_string();
            row.creation = dt2;
            row.last_login = Some(dt3);
            row.store(&pool).await.unwrap();

            // load
            let mut row = DbDataRow::new(1);
            row.load(&pool).await.unwrap();
            assert_eq!(row.rowid, 1);
            assert_eq!(row.user, Some(46));
            assert_eq!(row.steam_id, "NewSomeSteam64GUID".to_string());
            assert_eq!(row.creation, dt2.clone());
            assert_eq!(row.last_login, Some(dt3.clone()));
        }
    }

    mod item {
        use test_log::test;
        use super::*;

        #[test(tokio::test)]
        async fn test() {
            let earlier: DateTime<Utc> = Utc::now();
            let pool = get_pool().await;
            let item = create_new_item(&pool).await;
            assert!(item.creation().await > earlier);
        }
    }

    mod table {
        use test_log::test;
        use super::*;

        #[test(tokio::test)]
        async fn test_new() {
            let tbl = get_table_interface().await;

            let item0 = tbl.item_by_steam_id("SteamId1", false).await;
            assert!(item0.is_none());

            let item1 = tbl.item_by_steam_id("SteamId1", true).await.unwrap();
            assert!(item1.id().await > 0);
            let item2 = tbl.item_by_steam_id("SteamId1", true).await.unwrap();
            assert_eq!(item1.id().await, item2.id().await);
        }
    }
}
