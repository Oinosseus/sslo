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
use crate::db2::members::MembersDbData;

#[derive(PartialEq)]
pub enum Activity {
    None = 0,
    Obsolete = 1,
    Recent = 2,
}

impl Activity {
    pub fn from_date_time(last_activity: Option<DateTime<Utc>>) -> Self {

        // determine date of recent activity
        let t_now = chrono::Utc::now();
        let t_recent = t_now.sub(chrono::Duration::days(90));

        // return enum
        match last_activity {
            None => Activity::None,
            Some(t_login) if t_login < t_recent => Activity::Obsolete,
            Some(_) => Activity::Recent,
        }
    }
}

pub struct UserActivity {
    driving_activity: Activity,
    login_activity: Activity,
}

impl UserActivity {

    pub fn new() -> Self {
        Self {
            driving_activity: Activity::None,
            login_activity: Activity::None
        }
    }

    pub fn label(&self) -> &'static str {
        match self.login_activity {
            Activity::None => {
                match self.driving_activity {
                    Activity::None => {"Wildcard Pedestrian"}
                    Activity::Obsolete => {"Wildcard Veteran"}
                    Activity::Recent => {"Wildcard Driver"}
                }
            },
            Activity::Obsolete => {
                match self.driving_activity {
                    Activity::None => {"Ghost Pedestrian"}
                    Activity::Obsolete => {"Ghost Veteran"}
                    Activity::Recent => {"Ghost Driver"}
                }
            },
            Activity::Recent => {
                match self.driving_activity {
                    Activity::None => {"League Pedestrian"}
                    Activity::Obsolete => {"League Veteran"}
                    Activity::Recent => {"League Driver"}
                }
            },
        }
    }
}


#[derive(PartialEq, Clone)]
#[derive(sqlx::Type)]
#[derive(Debug)]
#[repr(u32)]
pub enum PromotionAuthority {

    /// Only executing his promotion (cannot promote others)
    Executing = 0,

    /// Can also promote other users (up to one level below)
    Chief = 1,
}


#[derive(PartialEq, Clone)]
#[derive(sqlx::Type)]
#[derive(Debug)]
#[repr(u32)]
pub enum PromotionLevel {
    /// no further user rights
    None = 0,

    /// graceful server control
    Steward = 1,

    /// force server control, update downloads
    Marshal = 2,

    /// schedule races
    Officer = 3,

    /// correct results, pronounce penalties
    Commissar = 4,

    /// manage series, edit presets
    Director = 5,

    /// almost all permissions (except root)
    Admin = 6,
}


pub struct Promotion {
    pub level: PromotionLevel,
    pub authority: PromotionAuthority,
}

impl Promotion {

    pub fn new(level: PromotionLevel, authority: PromotionAuthority) -> Self {
        Self { level, authority }
    }

    pub fn new_lowest() -> Self {
        Self {
            level: PromotionLevel::None,
            authority: PromotionAuthority::Executing,
        }
    }

    pub fn label(&self) -> &'static str {
        match self.level {
            PromotionLevel::None => "",
            PromotionLevel::Steward => match self.authority {
                PromotionAuthority::Executing => {"Executing Steward"}
                PromotionAuthority::Chief => {"Chief Steward"}
            },
            PromotionLevel::Marshal => match self.authority {
                PromotionAuthority::Executing => {"Executing Marshal"}
                PromotionAuthority::Chief => {"Chief Marshal"}
            },
            PromotionLevel::Officer => match self.authority {
                PromotionAuthority::Executing => {"Executing Officer"}
                PromotionAuthority::Chief => {"Chief Officer"}
            },
            PromotionLevel::Commissar => match self.authority {
                PromotionAuthority::Executing => {"Executing Commissar"}
                PromotionAuthority::Chief => {"Chief Commissar"}
            },
            PromotionLevel::Director => match self.authority {
                PromotionAuthority::Executing => {"Executing Director"}
                PromotionAuthority::Chief => {"Chief Director"}
            },
            PromotionLevel::Admin => match self.authority {
                PromotionAuthority::Executing => {"Executing Administrator"}
                PromotionAuthority::Chief => {"Chief Administrator"}
            },
        }
    }

    pub fn symbol(&self) -> &'static str {
        todo!()
    }
}

#[derive(sqlx::FromRow, Clone)]
struct DbDataRow {
    pub(super) rowid: i64,
    pub(super) name: String,
    pub(super) promotion_authority: PromotionAuthority,
    pub(super) promotion_level: PromotionLevel,
    pub(super) last_lap: Option<DateTime<Utc>>,
    pub(super) last_login: Option<DateTime<Utc>>,
    pub(super) password: Option<String>,
    pub(super) password_last_usage: Option<DateTime<Utc>>,
    pub(super) password_last_useragent: Option<String>,
}

impl DbDataRow {

    /// Create a new (empty/default) data row
    fn new(rowid: i64) -> Self {
        debug_assert!(rowid >= 0);
        Self {
            rowid,
            name: "".to_string(),
            promotion_authority: PromotionAuthority::Executing,
            promotion_level: PromotionLevel::None,
            last_lap: None,
            last_login: None,
            password: None,
            password_last_usage: None,
            password_last_useragent: None,
        }
    }

    /// directly retrieve an item from database by email address
    async fn from_email(email: &str, pool: &SqlitePool) -> Result<Self, SsloError> {
        return match sqlx::query_as::<Sqlite, DbDataRow>(concat!("SELECT rowid,* FROM ", tablename!(), " WHERE email LIKE $1 LIMIT 2;"))
            .bind(email)
            .fetch_one(pool)
            .await {
            Ok(row) => {
                Ok(row)
            },
            Err(sqlx::Error::RowNotFound) => {
                Err(SsloError::DatabaseDataNotFound(tablename!(), "email", email.to_string()))
            },
            Err(e) => {
                return Err(SsloError::DatabaseSqlx(e));
            }
        };
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
                "(name,\
                  promotion_authority,\
                  promotion_level,\
                  last_lap,\
                  last_login,\
                  password,\
                  password_last_usage,\
                  password_last_useragent) \
                  VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING rowid;"))
            },
            _ => {
                sqlx::query(concat!("UPDATE ", tablename!(), " SET \
                                   name=$1,\
                                   promotion_authority=$2,\
                                   promotion_level=$3,\
                                   last_lap=$4,\
                                   last_login=$5,\
                                   password=$6,\
                                   password_last_usage=$7,\
                                   password_last_useragent=$8 \
                                   WHERE rowid=$9;"))
            }
        };

        // bind values
        query = query.bind(&self.name)
            .bind(&self.promotion_authority)
            .bind(&self.promotion_level)
            .bind(&self.last_lap)
            .bind(&self.last_login)
            .bind(&self.password)
            .bind(&self.password_last_usage)
            .bind(&self.password_last_useragent);
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
        format!("{}(rowid={};name={})", tablename!(), self.rowid, self.name)
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

    pub async fn id(&self) -> i64 {
        self.0.read().await.row.rowid
    }


    pub async fn name(&self) -> String {
        self.0.read().await.row.name.clone()
    }

    pub async fn html_name(&self) -> String {
        let mut html = String::new();
        html_escape::encode_safe_to_string(&self.0.read().await.row.name, &mut html);
        return html;
    }

    pub async fn set_name(self: &mut Self, name: String) -> Result<(), SsloError> {
        let mut data = self.0.write().await;
        let name = name.trim().to_string();
        let display_before = data.row.display();
        data.row.name = name;
        let display_after = data.row.display();
        log::info!("Change {} to {}", display_before, display_after);
        match data.pool.clone() {
            None => Ok(()),
            Some(pool) => data.row.store(&pool).await
        }
    }

    pub async fn activity(&self) -> UserActivity {
        let data = self.0.read().await;
        UserActivity {
            driving_activity: Activity::from_date_time(data.row.last_lap),
            login_activity: Activity::from_date_time(data.row.last_login),
        }
    }

    pub async fn promotion(&self) -> Promotion {
        let data = self.0.read().await;
        Promotion::new(data.row.promotion_level.clone(), data.row.promotion_authority.clone())
    }
    pub async fn set_promotion(&mut self, promotion: Promotion) {
        let mut item_data = self.0.write().await;
        item_data.row.promotion_level = promotion.level;
        item_data.row.promotion_authority = promotion.authority;
        if let Some(pool) = item_data.pool.clone() {
            if let Err(e) = item_data.row.store(&pool).await {
                log::error!("Failed to set promotion for {}: {}", item_data.row.display(), e);
            };
        }
    }

    pub async fn last_lap(&self) -> OptionalDateTime { OptionalDateTime::new(self.0.read().await.row.last_lap) }
    pub async fn set_last_lap(self: &mut Self, last_lap: DateTime<Utc>) {
        let mut data = self.0.write().await;
        data.row.last_lap = Some(last_lap);
        if let Some(pool) = data.pool.clone() {
            if let Err(e) = data.row.store(&pool).await {
                log::error!("Failed to set last lap for {}: {}", data.row.display(), e);
            }
        }
    }

    pub async fn last_login(&self) -> OptionalDateTime { OptionalDateTime::new(self.0.read().await.row.last_login) }
    pub async fn set_last_login(self: &mut Self, last_login: DateTime<Utc>) {
        let mut data = self.0.write().await;
        data.row.last_login = Some(last_login);
        if let Some(pool) = data.pool.clone() {
            if let Err(e) = data.row.store(&pool).await {
                log::error!("Failed to set last login for {}: {}", data.row.display(), e);
            }
        }
    }

    /// Consume a cleartext password, and store encrypted
    /// This checks if the current password is valid
    pub async fn update_password(&mut self, old_password: Option<String>, new_password: Option<String>) -> bool {
        let mut data = self.0.write().await;

        // verify old password
        if let Some(old_password_encrypted) = data.row.password.as_ref() {
            if let Some(old_password_decrypted) = old_password {
                match argon2::verify_encoded(old_password_encrypted, &old_password_decrypted.into_bytes()) {
                    Ok(true) => {},
                    Ok(false) => {
                        log::warn!("deny update password, because invalid old password given for {}", data.row.display());
                        return false;
                    },
                    Err(e) => {
                        log::error!("Argon2 failure at verifying passwords: {}", e);
                        return false;
                    }
                }
            } else {
                log::warn!("deny update password, because no old password given for {}", data.row.display());
                return false;
            }
        }

        // encrypt new password
        let mut new_password_encrypted: Option<String> = None;
        if let Some(some_new_password) = new_password {
            let mut salt: Vec<u8> = vec![0u8; 64];
            rand::thread_rng().fill_bytes(&mut salt);
            new_password_encrypted = match argon2::hash_encoded(&some_new_password.into_bytes(), &salt, &argon2::Config::default()) {
                Ok(p) => Some(p),
                Err(e) => {
                    log::error!("Argon2 failed to encrypt password for {}: {}", data.row.display(), e);
                    return false;
                }
            };
        }

        // update password
        data.row.password = new_password_encrypted;
        data.row.password_last_usage = None;
        data.row.password_last_useragent = None;
        if let Some(pool) = data.pool.clone() {
            if let Err(e) = data.row.store(&pool).await {
                log::error!("failed to store updated password for {}: {}", data.row.display(), e);
                return false;
            }
        }

        log::info!("password updated for user {}", data.row.display());
        return true;
    }

    /// Consumes a cleartext password
    pub async fn verify_password(&self, password: String, user_agent: String) -> bool {

        {   // separate scope with read-lock for quick return at verification fail
            let data = self.0.read().await;
            if let Some(old_password_encrypted) = data.row.password.as_ref() {
                match argon2::verify_encoded(old_password_encrypted, &password.into_bytes()) {
                    Ok(true) => {}
                    Ok(false) => {
                        return false;
                    }
                    Err(e) => {
                        log::error!("Argon2 failure at verifying passwords for {}: {}", data.row.display(), e);
                        return false;
                    }
                }
            } else {
                log::warn!("deny verifying password, because no password set for {}", data.row.display());
                return false;
            }
        }

        // update usage
        let mut data = self.0.write().await;
        data.row.password_last_usage = Some(Utc::now());
        data.row.password_last_useragent = Some(user_agent);
        if let Some(pool) = data.pool.clone() {
            if let Err(e) = data.row.store(&pool).await {
                log::error!("failed to update password usage for {}: {}", data.row.display(), e);
                return false;
            }
        }

        // done
        log::info!("successful password verification for {}", data.row.display());
        true
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
        sqlx::migrate!("../rsc/db_migrations/league_members").run(&pool).await.unwrap();
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

        #[test(tokio::test)]
        async fn promotion() {

            // create item
            let pool = super::get_pool().await;
            let mut item = create_new_item(&pool.clone()).await;

            // modify item (ne before, eq after)
            let prom = item.promotion().await;
            assert_ne!(prom.level, PromotionLevel::Marshal);
            assert_ne!(prom.authority, PromotionAuthority::Chief);
            item.set_promotion(Promotion::new(PromotionLevel::Marshal, PromotionAuthority::Chief)).await;
            let prom = item.promotion().await;
            assert_eq!(prom.level, PromotionLevel::Marshal);
            assert_eq!(prom.authority, PromotionAuthority::Chief);

            // check if stored into db_obsolete correctly
            let item = load_item_from_db(item.id().await, &pool).await;
            let prom = item.promotion().await;
            assert_eq!(prom.level, PromotionLevel::Marshal);
            assert_eq!(prom.authority, PromotionAuthority::Chief);
        }

        #[test(tokio::test)]
        async fn last_lap() {

            // create item
            let pool = super::get_pool().await;
            let mut item = create_new_item(&pool.clone()).await;

            // prepare test data
            let dt: DateTime<Utc> = DateTime::parse_from_rfc3339("1001-01-01T01:01:01.1111+01:00").unwrap().into();

            // modify item (ne before, eq after)
            assert_eq!(item.last_lap().await.raw(), &None);
            item.set_last_lap(dt).await;
            assert_eq!(item.last_lap().await.raw(), &Some(dt));

            // check if stored into db_obsolete correctly
            let item = load_item_from_db(item.id().await, &pool).await;
            assert_eq!(item.last_lap().await.raw(), &Some(dt));
        }

        #[test(tokio::test)]
        async fn password() {

            // create item
            let pool = super::get_pool().await;
            let mut item = create_new_item(&pool.clone()).await;

            // set password
            assert!(item.update_password(None, Some("unsecure_test_password".to_string())).await);
            assert!(item.verify_password("unsecure_test_password".to_string(), "unit test".to_string()).await);

            // check if stored into db_obsolete correctly
            let mut item = load_item_from_db(item.id().await, &pool).await;
            assert!(item.verify_password("unsecure_test_password".to_string(), "unit test".to_string()).await);

            // update without old password must fail
            assert!(!item.update_password(None, Some("unsecure_updated_test_password".to_string())).await);

            // update password
            assert!(item.update_password(Some("unsecure_test_password".to_string()), Some("unsecure_updated_test_password".to_string())).await);

            // check if stored into db_obsolete correctly
            let item = load_item_from_db(item.id().await, &pool).await;
            assert!(item.verify_password("unsecure_updated_test_password".to_string(), "unit test".to_string()).await);

            // verify wrong password must fail
            assert!(!item.verify_password("foobar".to_string(), "unit test".to_string()).await);
        }

    }

    mod row {
        use chrono::{DateTime, Utc};
        use super::*;
        use test_log::test;

        #[test(tokio::test)]
        async fn new_defaults() {
            let row = DbDataRow::new(33);
            assert_eq!(row.rowid, 33);
            assert_eq!(row.name, "".to_string());
            assert_eq!(row.promotion_authority, PromotionAuthority::Executing);
            assert_eq!(row.promotion_level, PromotionLevel::None);
            assert_eq!(row.last_lap, None);
            assert_eq!(row.last_login, None);
            assert_eq!(row.password, None);
            assert_eq!(row.password_last_usage, None);
            assert_eq!(row.password_last_useragent, None);
        }

        /// Testing load and store (insert+update)
        #[test(tokio::test)]
        async fn load_store() {
            let pool = super::get_pool().await;

            // define some UTC times
            let dt1: DateTime<Utc> = DateTime::parse_from_rfc3339("1001-01-01T01:01:01.1111+01:00").unwrap().into();
            let dt2: DateTime<Utc> = DateTime::parse_from_rfc3339("2002-02-02T02:02:02.2222+02:00").unwrap().into();
            let dt3: DateTime<Utc> = DateTime::parse_from_rfc3339("3003-03-03T03:03:03.3333+03:00").unwrap().into();
            let dt4: DateTime<Utc> = DateTime::parse_from_rfc3339("4004-04-04T04:04:04.4444+04:00").unwrap().into();
            let dt5: DateTime<Utc> = DateTime::parse_from_rfc3339("5005-05-05T05:05:05.5555+05:00").unwrap().into();

            // store (insert)
            let mut row = DbDataRow::new(0);
            row.name = "RowName".to_string();
            row.promotion_authority = PromotionAuthority::Chief;
            row.promotion_level = PromotionLevel::Commissar;
            row.last_lap = Some(dt1.clone());
            row.last_login = Some(dt2.clone());
            row.password = Some("IAmThePassword".to_string());
            row.password_last_usage = Some(dt5.clone());
            row.password_last_useragent = Some("IAmTheUserAgent".to_string());
            row.store(&pool).await.unwrap();

            // load
            let mut row = DbDataRow::new(1);
            row.load(&pool).await.unwrap();
            assert_eq!(row.rowid, 1);
            assert_eq!(row.name, "RowName".to_string());
            assert_eq!(row.promotion_authority, PromotionAuthority::Chief);
            assert_eq!(row.promotion_level, PromotionLevel::Commissar);
            assert_eq!(row.last_lap, Some(dt1.clone()));
            assert_eq!(row.last_login, Some(dt2.clone()));
            assert_eq!(row.password, Some("IAmThePassword".to_string()));
            assert_eq!(row.password_last_usage, Some(dt5.clone()));
            assert_eq!(row.password_last_useragent, Some("IAmTheUserAgent".to_string()));

            // store (update)
            let mut row = DbDataRow::new(1);
            row.name = "RowNameNew".to_string();
            row.promotion_authority = PromotionAuthority::Executing;
            row.promotion_level = PromotionLevel::Admin;
            row.last_lap = Some(dt2.clone());
            row.last_login = Some(dt3.clone());
            row.password = Some("IAmThePasswordNew".to_string());
            row.password_last_usage = Some(dt1.clone());
            row.password_last_useragent = Some("IAmTheUserAgentNew".to_string());
            row.store(&pool).await.unwrap();

            // load
            let mut row = DbDataRow::new(1);
            row.load(&pool).await.unwrap();
            assert_eq!(row.rowid, 1);
            assert_eq!(row.name, "RowNameNew".to_string());
            assert_eq!(row.promotion_authority, PromotionAuthority::Executing);
            assert_eq!(row.promotion_level, PromotionLevel::Admin);
            assert_eq!(row.last_lap, Some(dt2.clone()));
            assert_eq!(row.last_login, Some(dt3.clone()));
            assert_eq!(row.password, Some("IAmThePasswordNew".to_string()));
            assert_eq!(row.password_last_usage, Some(dt1.clone()));
            assert_eq!(row.password_last_useragent, Some("IAmTheUserAgentNew".to_string()));
        }
    }
}
