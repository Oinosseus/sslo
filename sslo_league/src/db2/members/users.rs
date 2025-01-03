macro_rules! tablename {
    () => { "users" };
}

use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;
use crate::user_grade::{Promotion, PromotionAuthority};
use chrono::{DateTime, Utc};
use sqlx::{Sqlite, SqlitePool};
use rand::RngCore;
use sslo_lib::error::SsloError;
use sslo_lib::token::{Token, TokenType};

#[derive(sqlx::FromRow, Clone)]
struct DbDataRow {
    pub(super) rowid: i64,
    pub(super) name: String,
    pub(super) promotion_authority: PromotionAuthority,
    pub(super) promotion: Promotion,
    pub(super) last_lap: Option<DateTime<Utc>>,
    pub(super) email: Option<String>,
    pub(super) email_token: Option<String>,
    pub(super) email_token_creation: Option<DateTime<Utc>>,
    pub(super) email_token_consumption: Option<DateTime<Utc>>,
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
            promotion: Promotion::None,
            last_lap: None,
            email: None,
            email_token: None,
            email_token_creation: None,
            email_token_consumption: None,
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
                  promotion,\
                  last_lap,\
                  email,\
                  email_token,\
                  email_token_creation,\
                  email_token_consumption,\
                  password,\
                  password_last_usage,\
                  password_last_useragent) \
                  VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING rowid;"))
            },
            _ => {
                sqlx::query(concat!("UPDATE ", tablename!(), " SET \
                                   name=$1,\
                                   promotion_authority=$2,\
                                   promotion=$3,\
                                   last_lap=$4,\
                                   email=$5,\
                                   email_token=$6,\
                                   email_token_creation=$7,\
                                   email_token_consumption=$8,\
                                   password=$9,\
                                   password_last_usage=$10,\
                                   password_last_useragent=$11 \
                                   WHERE rowid=$12;"))
            }
        };

        // bind values
        query = query.bind(&self.name)
            .bind(&self.promotion_authority)
            .bind(&self.promotion)
            .bind(&self.last_lap)
            .bind(&self.email)
            .bind(&self.email_token)
            .bind(&self.email_token_creation)
            .bind(&self.email_token_consumption)
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
}


/// The actual data of an item that is shared by Arc<RwLock<ItemData>>
struct UserItemData {
    pool: SqlitePool,
    row: DbDataRow,
}

impl UserItemData {
    fn new(pool: &SqlitePool, row: DbDataRow) -> Arc<RwLock<UserItemData>> {
        Arc::new(RwLock::new(Self {
            pool: pool.clone(),
            row,
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

    pub async fn id(&self) -> i64 {
        self.0.read().await.row.rowid
    }


    pub async fn name(&self) -> String {
        self.0.read().await.row.name.clone()
    }

    pub async fn set_name(self: &mut Self, name: String) -> Result<(), SsloError> {
        let mut data = self.0.write().await;
        data.row.name = name;
        let pool = data.pool.clone();
        data.row.store(&pool).await
    }

    pub async fn promotion_authority(&self) -> PromotionAuthority { self.0.read().await.row.promotion_authority.clone() }
    pub async fn promotion(&self) -> Promotion { self.0.read().await.row.promotion.clone() }
    pub async fn set_promotion(&mut self, promotion: Promotion, authority: PromotionAuthority) {
        let mut item_data = self.0.write().await;
        item_data.row.promotion = promotion;
        item_data.row.promotion_authority = authority;
        let pool = item_data.pool.clone();
        if let Err(e) = item_data.row.store(&pool).await {
            log::error!("Failed to set promotion: {}", e);
        };
    }

    pub async fn last_lap(&self) -> Option<DateTime<Utc>> { self.0.read().await.row.last_lap }
    pub async fn set_last_lap(self: &mut Self, last_lap: DateTime<Utc>) {
        let mut data = self.0.write().await;
        data.row.last_lap = Some(last_lap);
        let pool = data.pool.clone();
        if let Err(e) = data.row.store(&pool).await {
            log::error!("Failed to set last lap: {}", e);
        };
    }

    /// Returns email address (if correctly confirmed)
    pub async fn email(&self) -> Option<String> {
        let now = Utc::now();
        let item_data = self.0.read().await;

        // ensure email is set
        let email = match item_data.row.email.as_ref() {
            Some(x) => x,
            None => return None,
        };

        // ensure email is verified
        match item_data.row.email_token_consumption.as_ref() {
            Some(t) if t > &now => {
                log::error!("Token creation/consumption time mismatch for rowid={}, email='{}', consumption='{}'",
                            item_data.row.rowid, email, t);
                return None;
            },
            Some(t) => {
                t
            },
            None => {
                log::warn!("hide email, because token not verified for user rowid={}, email={:?}",
                    item_data.row.rowid, item_data.row.email);
                return None;
            }
        };

        return Some(email.clone());
    }

    /// Returns a token, that must be sent to the customer for confirmation
    pub async fn set_email(&mut self, email: String) -> Option<String> {
        // let email = email.to_lowercase();  // convention to store only lower-case
        let mut item_data = self.0.write().await;

        // check for timeout since last token creation
        let time_now = Utc::now();
        let time_token_outdated = time_now.clone()
            .checked_add_signed(chrono::TimeDelta::hours(-1))
            .unwrap();  // subtracting one hour cannot fail, theoretically
        if let Some(token_creation) = item_data.row.email_token_creation {
            if token_creation > time_token_outdated {               // token is still valid
                if item_data.row.email_token_consumption.is_none() {    // token is not used, yet
                    log::warn!("Not generating new email login token for user {}:'{}' because last token is still active.", item_data.row.rowid, email);
                    return None;
                }
            }
        }

        // check for unique email
        let pool = item_data.pool.clone();
        match DbDataRow::from_email(&email, &pool).await {
            Ok(row) => {
                if row.rowid != item_data.row.rowid {
                    log::warn!("reject assigning email '{}' because duplicate at rowid={}", &email, row.rowid);
                    return None;
                }
            },
            Err(e) if e.is_db_not_found_type() => {},
            Err(e) => {
                log::error!("failed to email uniqueness for email = '{}': {}", &email, e);
                return None;
            },
        }

        // generate new email_token
        let token = match Token::generate(TokenType::Strong) {
            Ok(t) => t,
            Err(e) => {
                log::error!("Could not generate new token: {}", e);
                return None;
            }
        };

        // update data
        item_data.row.email = Some(email);
        item_data.row.email_token = Some(token.encrypted);
        item_data.row.email_token_creation = Some(time_now);
        item_data.row.email_token_consumption = None;
        return match item_data.row.store(&pool).await {
            Ok(_) => Some(token.decrypted),
            Err(e) => {
                log::error!("failed to store new email token for user rowid={} into db: {}", item_data.row.rowid, e);
                None
            }
        };
    }

    pub async fn verify_email(&self, token_decrypted: String) -> bool {
        let mut item_data = self.0.write().await;
        let time_now = Utc::now();
        let time_token_outdated = time_now.clone()
            .checked_add_signed(chrono::TimeDelta::hours(-1))
            .unwrap();  // subtracting one hour cannot fail, theoretically

        // ensure encrypted token is set
        let token_encrypted = match item_data.row.email_token.as_ref() {
            Some(x) => x,
            None => {
                log::warn!("deny email verification because no email token set for user rowid={}; email={:?}",
                    item_data.row.rowid, item_data.row.email);
                return false;
            },
        };

        // ensure token is not already consumed
        if let Some(consumption_time) = item_data.row.email_token_consumption.as_ref() {
            log::warn!("deny email token validation for user rowid={}, email={:?}, because token already consumed at {}",
                        item_data.row.rowid, item_data.row.email, consumption_time);
            return false;
        }

        // ensure creation time is not outdated
        match item_data.row.email_token_creation.as_ref() {
            None => {
                log::error!("deny email verification, because no token-creation time set for user rowid={}; email={:?}",
                    item_data.row.rowid, item_data.row.email);
                return false;
            },
            Some(token_creation) => {
                if token_creation < &time_token_outdated {
                    log::warn!("deny email verification, because token is outdated since {} for user rowid={}; email={:?}",
                                        time_token_outdated, item_data.row.rowid, item_data.row.email);
                    return false;
                }
            },
        }

        // verify token
        if !sslo_lib::token::Token::new(token_decrypted, token_encrypted.clone()).verify() {
            log::warn!("deny email verification because token verification failed for rowid={}, email={:?}",
                item_data.row.rowid, item_data.row.email);
            return false;
        }

        // update email_token_consumption
        item_data.row.email_token = None;  // reset for security
        item_data.row.email_token_consumption = Some(time_now);
        let pool = item_data.pool.clone();
        return match item_data.row.store(&pool).await {
            Ok(_) => true,
            Err(e) => {
                log::error!("failed to store verified email token for rowid={}, email={:?}: {}",
                item_data.row.rowid, item_data.row.email, e);
                false
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
                        log::warn!("deny update password, because invalid old password given for rowid={}", data.row.rowid);
                        return false;
                    },
                    Err(e) => {
                        log::error!("Argon2 failure at verifying passwords: {}", e);
                        return false;
                    }
                }
            } else {
                log::warn!("deny update password, because no old password given for rowid={}", data.row.rowid);
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
                    log::error!("Argon2 failed to encrypt password: {}", e);
                    return false;
                }
            };
        }

        // update password
        data.row.password = new_password_encrypted;
        data.row.password_last_usage = None;
        data.row.password_last_useragent = None;
        let pool = data.pool.clone();
        if let Err(e) = data.row.store(&pool).await {
            log::error!("failed to store updated password for rowid={}: {}", data.row.rowid, e);
            return false;
        }

        log::info!("password updated for rowid={}", data.row.rowid);
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
                        log::error!("Argon2 failure at verifying passwords: {}", e);
                        return false;
                    }
                }
            } else {
                log::warn!("deny verifying password, because no password set for rowid={}", data.row.rowid);
                return false;
            }
        }

        // update usage
        let mut data = self.0.write().await;
        data.row.password_last_usage = Some(Utc::now());
        data.row.password_last_useragent = Some(user_agent);
        let pool = data.pool.clone();
        if let Err(e) = data.row.store(&pool).await {
            log::error!("failed to update password usage for rowid={}: {}", data.row.rowid, e);
            return false;
        }

        return true;
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
                    log::error!("Could not create a new user: {}", e);
                    return None;
                }
            }
        }

        // update cache
        let mut tbl_data = self.0.write().await;
        let item_data = UserItemData::new(&tbl_data.pool, row);
        let item = UserItem::new(item_data.clone());
        tbl_data.item_cache.insert(item.id().await, item_data);
        return Some(item);
    }

    /// Get an item
    /// This first tries to load the item from cache,
    /// and secondly load it from the database.
    pub async fn user_by_id(&self, id: i64) -> Option<UserItem> {

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
            let item_data = UserItemData::new(&tbl_data.pool, row);
            let item = UserItem::new(item_data.clone());
            tbl_data.item_cache.insert(id, item_data);
            return Some(item);
        }
    }


    /// Search the database for an email and then return the item
    /// The search is case-insensitive,
    /// this is not cached -> expensive
    pub async fn user_by_email(&self, email: &str) -> Option<UserItem> {
        let pool: SqlitePool;
        {   // scoped lock to call user_by_id() later
            let data = self.0.read().await;
            pool = data.pool.clone();
        }
        let row = match DbDataRow::from_email(email, &pool).await {
            Ok(row) => {row},
            Err(e) => {
                if e.is_db_not_found_type() {
                    log::warn!("{}", e);
                } else {
                    log::error!("{}", e.to_string());
                }
                return None;
            },
        };
        return self.user_by_id(row.rowid).await;
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

            // append items to db
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

        #[test(tokio::test)]
        async fn user_by_email() {
            let tbl = super::get_table_interface().await;
            let mut user = tbl.create_new_user().await.unwrap();
            let token = user.set_email("a.B@c.de".to_string()).await.unwrap();
            assert!(user.verify_email(token).await);

            // retrieve item
            let user = tbl.user_by_email("a.B@c.de").await.unwrap();
            assert_eq!(user.email().await.unwrap(), "a.B@c.de".to_string());

            // check case insensitivity
            let user = tbl.user_by_email("a.b@c.de").await.unwrap();
            assert_eq!(user.email().await.unwrap(), "a.B@c.de".to_string());
        }

        #[test(tokio::test)]
        async fn duplicated_email() {
            let tbl = super::get_table_interface().await;

            // create a user, set email
            let mut user = tbl.create_new_user().await.unwrap();
            let token = user.set_email("a.B@c.de".to_string()).await.unwrap();

            // create another user, with same email -> should fail
            let mut user = tbl.create_new_user().await.unwrap();
            assert!(user.set_email("a.b@c.de".to_string()).await.is_none());
        }
    }

    mod item {
        use chrono::{DateTime, Utc};
        use sqlx::SqlitePool;
        use super::super::*;
        use crate::user_grade::{Promotion, PromotionAuthority};
        use test_log::test;

        async fn create_new_item(pool: &SqlitePool) -> UserItem {
            let row = DbDataRow::new(0);
            let data = UserItemData::new(pool, row);
            UserItem::new(data)
        }

        async fn load_item_from_db(id: i64, pool: &SqlitePool) -> UserItem {
            let mut row = DbDataRow::new(id);
            row.load(pool).await.unwrap();
            let data = UserItemData::new(&pool, row);
            UserItem::new(data)
        }

        /// test item generation and property access
        #[test(tokio::test)]
        async fn new_item() {
            let pool = super::get_pool().await;

            // create item
            let row = DbDataRow::new(0);
            let data = UserItemData::new(&pool, row);
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
            item.set_name("Ronny".to_string()).await.unwrap();
            assert_eq!(item.id().await, 1);
            assert_eq!(item.name().await, "Ronny");

            // check if arrived in database
            let item = load_item_from_db(1, &pool).await;
            assert_eq!(item.id().await, 1);
            assert_eq!(item.name().await, "Ronny");
        }

        #[test(tokio::test)]
        async fn promotion() {

            // create item
            let pool = super::get_pool().await;
            let mut item = create_new_item(&pool.clone()).await;

            // modify item (ne before, eq after)
            assert_ne!(item.promotion().await, Promotion::Marshal);
            assert_ne!(item.promotion_authority().await, PromotionAuthority::Chief);
            item.set_promotion(Promotion::Marshal, PromotionAuthority::Chief).await;
            assert_eq!(item.promotion().await, Promotion::Marshal);
            assert_eq!(item.promotion_authority().await, PromotionAuthority::Chief);

            // check if stored into db correctly
            let item = load_item_from_db(item.id().await, &pool).await;
            assert_eq!(item.promotion().await, Promotion::Marshal);
            assert_eq!(item.promotion_authority().await, PromotionAuthority::Chief);
        }

        #[test(tokio::test)]
        async fn last_lap() {

            // create item
            let pool = super::get_pool().await;
            let mut item = create_new_item(&pool.clone()).await;

            // prepare test data
            let dt: DateTime<Utc> = DateTime::parse_from_rfc3339("1001-01-01T01:01:01.1111+01:00").unwrap().into();

            // modify item (ne before, eq after)
            assert_eq!(item.last_lap().await, None);
            item.set_last_lap(dt).await;
            assert_eq!(item.last_lap().await, Some(dt));

            // check if stored into db correctly
            let item = load_item_from_db(item.id().await, &pool).await;
            assert_eq!(item.last_lap().await, Some(dt));
        }

        #[test(tokio::test)]
        async fn email() {

            // create item
            let pool = super::get_pool().await;
            let mut item = create_new_item(&pool.clone()).await;
            assert_eq!(item.email().await, None);

            // set email
            let email_token = item.set_email("a.b@c.de".to_string()).await.unwrap();
            assert_eq!(item.email().await, None);
            assert!(item.verify_email(email_token).await);
            assert_eq!(item.email().await, Some("a.b@c.de".to_string()));

            // check if stored into db correctly
            let item = load_item_from_db(item.id().await, &pool).await;
            assert_eq!(item.email().await, Some("a.b@c.de".to_string()));
        }

        #[test(tokio::test)]
        async fn password() {

            // create item
            let pool = super::get_pool().await;
            let mut item = create_new_item(&pool.clone()).await;
            assert_eq!(item.email().await, None);

            // set password
            assert!(item.update_password(None, Some("unsecure_test_password".to_string())).await);
            assert!(item.verify_password("unsecure_test_password".to_string(), "unit test".to_string()).await);

            // check if stored into db correctly
            let mut item = load_item_from_db(item.id().await, &pool).await;
            assert!(item.verify_password("unsecure_test_password".to_string(), "unit test".to_string()).await);

            // update without old password must fail
            assert!(!item.update_password(None, Some("unsecure_updated_test_password".to_string())).await);

            // update password
            assert!(item.update_password(Some("unsecure_test_password".to_string()), Some("unsecure_updated_test_password".to_string())).await);

            // check if stored into db correctly
            let item = load_item_from_db(item.id().await, &pool).await;
            assert!(item.verify_password("unsecure_updated_test_password".to_string(), "unit test".to_string()).await);

            // verify wrong password must fail
            assert!(!item.verify_password("foobar".to_string(), "unit test".to_string()).await);
        }

    }

    mod row {
        use chrono::{DateTime, Utc};
        use super::super::*;
        use crate::user_grade::{Promotion, PromotionAuthority};
        use test_log::test;

        #[test(tokio::test)]
        async fn new_defaults() {
            let row = DbDataRow::new(33);
            assert_eq!(row.rowid, 33);
            assert_eq!(row.name, "".to_string());
            assert_eq!(row.promotion_authority, PromotionAuthority::Executing);
            assert_eq!(row.promotion, Promotion::None);
            assert_eq!(row.last_lap, None);
            assert_eq!(row.email, None);
            assert_eq!(row.email_token, None);
            assert_eq!(row.email_token_creation, None);
            assert_eq!(row.email_token_consumption, None);
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

            // store (insert)
            let mut row = DbDataRow::new(0);
            row.name = "RowName".to_string();
            row.promotion_authority = PromotionAuthority::Chief;
            row.promotion = Promotion::Commissar;
            row.last_lap = Some(dt1.clone());
            row.email = Some("user@email.tld".into());
            row.email_token = Some("IAmAnEmailToken".to_string());
            row.email_token_creation = Some(dt2.clone());
            row.email_token_consumption = Some(dt3.clone());
            row.password = Some("IAmThePassword".to_string());
            row.password_last_usage = Some(dt4.clone());
            row.password_last_useragent = Some("IAmTheUserAgent".to_string());
            row.store(&pool).await.unwrap();

            // load
            let mut row = DbDataRow::new(1);
            row.load(&pool).await.unwrap();
            assert_eq!(row.rowid, 1);
            assert_eq!(row.name, "RowName".to_string());
            assert_eq!(row.promotion_authority, PromotionAuthority::Chief);
            assert_eq!(row.promotion, Promotion::Commissar);
            assert_eq!(row.last_lap, Some(dt1.clone()));
            assert_eq!(row.email, Some("user@email.tld".into()));
            assert_eq!(row.email_token, Some("IAmAnEmailToken".to_string()));
            assert_eq!(row.email_token_creation, Some(dt2.clone()));
            assert_eq!(row.email_token_consumption, Some(dt3.clone()));
            assert_eq!(row.password, Some("IAmThePassword".to_string()));
            assert_eq!(row.password_last_usage, Some(dt4.clone()));
            assert_eq!(row.password_last_useragent, Some("IAmTheUserAgent".to_string()));

            // store (update)
            let mut row = DbDataRow::new(1);
            row.name = "RowNameNew".to_string();
            row.promotion_authority = PromotionAuthority::Executing;
            row.promotion = Promotion::Admin;
            row.last_lap = Some(dt2.clone());
            row.email = Some("a.b@c.de".into());
            row.email_token = Some("IAmAnEmailTokenNew".to_string());
            row.email_token_creation = Some(dt3.clone());
            row.email_token_consumption = Some(dt4.clone());
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
            assert_eq!(row.promotion, Promotion::Admin);
            assert_eq!(row.last_lap, Some(dt2.clone()));
            assert_eq!(row.email, Some("a.b@c.de".into()));
            assert_eq!(row.email_token, Some("IAmAnEmailTokenNew".to_string()));
            assert_eq!(row.email_token_creation, Some(dt3.clone()));
            assert_eq!(row.email_token_consumption, Some(dt4.clone()));
            assert_eq!(row.password, Some("IAmThePasswordNew".to_string()));
            assert_eq!(row.password_last_usage, Some(dt1.clone()));
            assert_eq!(row.password_last_useragent, Some("IAmTheUserAgentNew".to_string()));
        }
    }
}
