use std::collections::HashMap;
use std::sync::{Arc, Weak};
use chrono::{DateTime, Utc};
use regex::Regex;
use sqlx::{FromRow, Sqlite, SqlitePool};
use tokio::sync::RwLock;
use sslo_lib::error::SsloError;
use sslo_lib::optional_date::OptionalDateTime;
use sslo_lib::token::{Token, TokenType};
use crate::db2::members::{MembersDbData, MembersDbInterface};
use crate::db2::members::users::UserItem;

macro_rules! tablename {
    () => { "email_accounts" };
}

#[derive(FromRow, Clone)]
struct DbDataRow {
    rowid: i64,
    user: Option<i64>,
    email: String,
    token: Option<String>,
    token_user: Option<i64>,
    token_creation: Option<DateTime<Utc>>,
    token_consumption: Option<DateTime<Utc>>,
}

impl DbDataRow {
    fn new(rowid: i64, email: String) -> Self {
        debug_assert!(rowid >= 0);
        Self {
            rowid,
            user: None,
            email,
            token: None,
            token_user: None,
            token_creation: None,
            token_consumption: None,
        }
    }

    async fn from_email(email: &str, pool: &SqlitePool) -> Result<Self, SsloError> {
        match sqlx::query_as::<Sqlite, DbDataRow>(concat!("SELECT rowid,* FROM ", tablename!(), " WHERE email LIKE $1 LIMIT 2;"))
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
                Err(SsloError::DatabaseSqlx(e))
            }
        }
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

    async fn load(self: &mut Self, pool: &SqlitePool) -> Result<(), SsloError> {
        match sqlx::query_as::<Sqlite, DbDataRow>(concat!("SELECT rowid,* FROM ", tablename!(), " WHERE rowid = $1 LIMIT 2;"))
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
        }
    }

    async fn store(self: &mut Self, pool: &SqlitePool) -> Result<(), SsloError> {

        // trim email
        self.email = self.email.trim().to_string().to_lowercase();

        // define query
        let mut query = match self.rowid {
            0 => {
                sqlx::query(concat!("INSERT INTO ", tablename!(),
                "(user,\
                  email,\
                  token,\
                  token_user, \
                  token_creation,\
                  token_consumption)\
                  VALUES ($1, $2, $3, $4, $5, $6) RETURNING rowid;"))
            },
            _ => {
                sqlx::query(concat!("UPDATE ", tablename!(), " SET \
                                   user=$1,\
                                   email=$2,\
                                   token=$3,\
                                   token_user=$4,\
                                   token_creation=$5,\
                                   token_consumption=$6 \
                                   WHERE rowid=$7;"))
            }
        };

        // bind values
        query = query.bind(&self.user)
            .bind(&self.email)
            .bind(&self.token)
            .bind(&self.token_user)
            .bind(&self.token_creation)
            .bind(&self.token_consumption);
        if self.rowid != 0 {
            query = query.bind(self.rowid);
        }

        // execute query
        let res = query.execute(pool).await?;
        if self.rowid == 0 {
            self.rowid = res.last_insert_rowid();
        }
        Ok(())
    }

    /// Returns a string that can be used for integrating this row into a log message
    fn display(&self) -> String {
        if let Some(user_id) = self.user {
            format!("{}(id={};email={};user-id={})", tablename!(), self.rowid, self.email, user_id)
        } else {
            format!("{}(id={};email={};user-id=None)", tablename!(), self.rowid, self.email)
        }
    }
}

struct EmailAccountItemData {
    pool: SqlitePool,
    row: DbDataRow,
    db_members: Weak<RwLock<MembersDbData>>,
}

impl EmailAccountItemData {

    fn new(pool: &SqlitePool, row: DbDataRow, db_members: Weak<RwLock<MembersDbData>>) -> Arc<RwLock<EmailAccountItemData>> {
        Arc::new(RwLock::new(Self {
            pool: pool.clone(),
            row,
            db_members,
        }))
    }
}

#[derive(Clone)]
pub struct EmailAccountItem(Arc<RwLock<EmailAccountItemData>>);

impl EmailAccountItem {
    fn new(item_data: Arc<RwLock<EmailAccountItemData>>) -> Self {
        Self(item_data)
    }

    /// Returns a string, that can be used in log messages
    pub async fn display(&self) -> String {
        self.0.read().await.row.display()
    }

    pub async fn id(&self) -> i64 {
        self.0.read().await.row.rowid
    }

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


    /// Returns true, if a user is assigned to this email
    pub async fn has_user(&self) -> bool {
        let data = self.0.read().await;
        data.row.user.is_some()
    }


    pub async fn set_user(&self, user: &UserItem) -> bool {
        let mut item_data = self.0.write().await;
        let pool = item_data.pool.clone();
        item_data.row.user = Some(user.id().await);
        match item_data.row.store(&pool).await {
            Ok(_) => true,
            Err(e) => {
                log::error!("Failed to set user for {}: {}", item_data.row.display(), e);
                false
            },
        }
    }

    pub async fn email(&self) -> String {
        self.0.read().await.row.email.clone()
    }

    /// Returns true, if the email token has been verified
    pub async fn is_verified(&self) -> bool {
        let now = Utc::now();
        let item_data = self.0.read().await;

        // ensure email is verified
        let token_creation = item_data.row.token_creation.as_ref();
        let token_consumption = item_data.row.token_consumption.as_ref();
        return match token_consumption {
            Some(some_token_consumption) if some_token_consumption > &now => {
                log::error!("Token creation/consumption time mismatch for {}, consumption='{}'",
                            item_data.row.display(), some_token_consumption);
                false
            },
            Some(some_token_consumption) => match token_creation {
                Some(some_token_creation) if some_token_creation > some_token_consumption => {
                    log::error!("Token creation/consumption time mismatch for {}, creation='{}', consumption='{}'",
                        item_data.row.display(),
                        some_token_creation,
                        some_token_consumption,
                    );
                    false
                },
                Some(_) => true,
                None => {
                    log::error!("Token consumed, but never created for {}, consumption='{}'",
                        item_data.row.display(),
                        some_token_consumption,
                    );
                    false
                }
            },
            None => false,
        }
    }

    /// Creates a new token, that shall be sent via email
    /// The token is not created, if an existing token is still pending.
    /// A token is pending until verified or until a timeout has passed.
    /// The token is stored into DB encrypted, the unencrypted token is returned.
    pub async fn create_token(&self, user: Option<&UserItem>) -> Option<String> {
        let mut item_data = self.0.write().await;
        let pool = item_data.pool.clone();
        let row_display = item_data.row.display();

        // check for timeout since last token creation
        let time_now = Utc::now();
        let time_token_outdated = time_now.clone()
            .checked_add_signed(chrono::TimeDelta::hours(-1))
            .unwrap();  // subtracting one hour cannot fail, theoretically
        if let Some(token_creation) = item_data.row.token_creation {
            if token_creation > time_token_outdated {  // token is still valid
                if item_data.row.token_consumption.is_none() {  // token is not used, yet
                    log::warn!("Not generating new email login token for {} because last token is still active.", &row_display);
                    return None;
                }
            }
        }

        // generate new email_token
        let token = match Token::generate(TokenType::Strong) {
            Ok(t) => t,
            Err(e) => {
                log::error!("Could not generate new token for {}: {}", row_display, e);
                return None;
            }
        };

        // update data and return
        item_data.row.token = Some(token.encrypted);
        item_data.row.token_user = match user {
            None => None,
            Some(u) => Some(u.id().await),
        };
        item_data.row.token_creation = Some(time_now);
        item_data.row.token_consumption = None;
        match item_data.row.store(&pool).await {
            Ok(_) => {
                log::info!("New token generated for {}", row_display);
                Some(token.decrypted)
            },
            Err(e) => {
                log::error!("failed to store new email token for {}: {}", row_display, e);
                None
            }
        }
    }

    /// This consumes a token which has been sent via email to verify a valid and owned email account
    pub async fn consume_token(&self, token: String) -> bool {
        let mut item_data = self.0.write().await;
        let row_display = item_data.row.display();
        let pool = item_data.pool.clone();
        let time_now = Utc::now();
        let time_token_outdated = time_now.clone()
            .checked_add_signed(chrono::TimeDelta::hours(-1))
            .unwrap();  // subtracting one hour cannot fail, theoretically

        // ensure encrypted token is set
        let token_encrypted = match item_data.row.token.as_ref() {
            Some(x) => x,
            None => {
                log::warn!("deny email verification because no email token set for user {}", row_display);
                return false;
            },
        };

        // ensure token is not already consumed
        if let Some(consumption_time) = item_data.row.token_consumption.as_ref() {
            log::warn!("deny email token validation for {}, because token already consumed at {}",
                        row_display, consumption_time);
            return false;
        }

        // ensure creation time is not outdated
        match item_data.row.token_creation.as_ref() {
            None => {
                log::error!("deny email verification, because no token-creation time set for user {}", row_display);
                return false;
            },
            Some(token_creation) => {
                if token_creation < &time_token_outdated {
                    log::warn!("deny email token verification for {}, because token is outdated since {}",
                                        row_display, time_token_outdated);
                    return false;
                }
            },
        }

        // verify token
        if !Token::new(token, token_encrypted.clone()).verify() {
            log::warn!("deny email verification because token verification failed for {}", row_display);
            return false;
        }

        // update email_token_consumption
        item_data.row.token = None;  // reset for security
        item_data.row.user = item_data.row.token_user;  // set requested user
        item_data.row.token_consumption = Some(time_now);
        if let Err(e) = item_data.row.store(&pool).await {
            log::error!("failed to store verified email token for{}: {}", row_display, e);
            return false;
        }

        // success
        log::info!("successfully verified email token for {}", row_display);
        true
    }

    pub async fn token_verification(&self) -> OptionalDateTime {
        let item_data = self.0.read().await;
        OptionalDateTime::new(item_data.row.token_consumption)
    }
}

pub(super) struct EmailAccountsTableData {
    pool: SqlitePool,
    item_cache_by_id: HashMap<i64, Arc<RwLock<EmailAccountItemData>>>,
    item_cache_by_email: HashMap<String, Arc<RwLock<EmailAccountItemData>>>,
    db_members: Weak<RwLock<MembersDbData>>,
}

impl EmailAccountsTableData {
    pub(super) fn new(pool: SqlitePool, db_members: Weak<RwLock<MembersDbData>>) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new( Self {
            pool,
            item_cache_by_id: HashMap::new(),
            item_cache_by_email: HashMap::new(),
            db_members,
        }))
    }
}

pub struct EmailAccountsTable(
    Arc<RwLock<EmailAccountsTableData>>
);

impl EmailAccountsTable {

    pub(super) fn new(data: Arc<RwLock<EmailAccountsTableData>>) -> Self {
        Self(data)
    }

    /// creates a new email account
    pub async fn create_account(&self, email: String) -> Option<EmailAccountItem> {
        let mut tbl_data = self.0.write().await;

        // check email
        let email = email.trim().to_string();
        let re = Regex::new("^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\\.[A-Za-z]{2,6}$").unwrap();  // modified from https://emailregex.com/
        if !re.is_match(&email) {
            log::warn!("Ignoring creating account with invalid email: '{}'", email);
            return None;
        }

        // create new row
        let mut row = DbDataRow::new(0, email);
        match row.store(&tbl_data.pool).await {
            Ok(_) => {},
            Err(e) => {
                log::error!("{}", e.to_string());
                return None;
            },
        }

        // create item
        let row_id = row.rowid;
        let row_email = row.email.clone();
        let item_data = EmailAccountItemData::new(&tbl_data.pool, row, tbl_data.db_members.clone());
        let item = EmailAccountItem::new(item_data.clone());
        tbl_data.item_cache_by_id.insert(row_id, item_data.clone());
        tbl_data.item_cache_by_email.insert(row_email, item_data);
        Some(item)
    }

    /// Get an item by id (may return unverified emails)
    pub async fn item_by_id(&self, id: i64) -> Option<EmailAccountItem> {

        // try cache hit
        {
            let tbl_data = self.0.read().await;
            if let Some(item_data) = tbl_data.item_cache_by_id.get(&id) {
                return Some(EmailAccountItem::new(item_data.clone()));
            }
        }

        // try loading from DB if not found in cache
        {
            let mut tbl_data = self.0.write().await;

            // load from db_obsolete
            let mut row = DbDataRow::new(id, "".to_string());
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
            let email = row.email.clone();
            let item_data = EmailAccountItemData::new(&tbl_data.pool, row, tbl_data.db_members.clone());
            let item = EmailAccountItem::new(item_data.clone());
            tbl_data.item_cache_by_id.insert(id, item_data.clone());
            tbl_data.item_cache_by_email.insert(email, item_data);
            Some(item)
        }
    }

    /// Get an item by email address (also returns unverified emails)
    pub async fn item_by_email_ignore_verification(&self, email: &str) -> Option<EmailAccountItem> {

        // try cache hit
        {
            let tbl_data = self.0.read().await;
            if let Some(item_data) = tbl_data.item_cache_by_email.get(email) {
                return Some(EmailAccountItem::new(item_data.clone()));
            }
        }

        // try loading from DB if not found in cache
        {
            let mut tbl_data = self.0.write().await;

            // load from db_obsolete
            let mut row = match DbDataRow::from_email(email, &tbl_data.pool).await {
                Ok(row) => row,
                Err(e) => {
                    if e.is_db_not_found_type() {
                        log::warn!("{}", e);
                    } else {
                        log::error!("{}", e.to_string());
                    }
                    return None;
                },
            };

            // create item
            let id = row.rowid;
            let item_data = EmailAccountItemData::new(&tbl_data.pool, row, tbl_data.db_members.clone());
            let item = EmailAccountItem::new(item_data.clone());
            tbl_data.item_cache_by_id.insert(id, item_data.clone());
            tbl_data.item_cache_by_email.insert(email.to_string(), item_data);
            Some(item)
        }
    }

    /// Get an item by email address (only verified emails)
    pub async fn item_by_email(&self, email: &str) -> Option<EmailAccountItem> {
        match self.item_by_email_ignore_verification(email).await {
            None => None,
            Some(item) => match item.is_verified().await {
                false => None,
                true => Some(item),
            },
        }
    }

    /// Get all email accounts that are associated to a certain user
    pub async fn items_by_user(&self, user: &UserItem) -> Vec<EmailAccountItem> {
        let mut tbl_data = self.0.write().await;
        let pool = tbl_data.pool.clone();
        let mut item_list: Vec<EmailAccountItem> = Vec::new();

        for row in DbDataRow::from_user(user, &pool).await.into_iter() {
            let rowid = row.rowid;
            let item = match tbl_data.item_cache_by_id.get(&rowid) {
                Some(item_data) => EmailAccountItem::new(item_data.clone()),
                None => {
                    let item_data = EmailAccountItemData::new(&pool, row, tbl_data.db_members.clone());
                    let item = EmailAccountItem::new(item_data.clone());
                    tbl_data.item_cache_by_id.insert(rowid, item_data.clone());
                    item
                }
            };
            item_list.push(item);
        }

        return item_list;
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

    async fn get_table_interface() -> EmailAccountsTable {
        let pool = get_pool().await;
        let tbl_data = EmailAccountsTableData::new(pool, Weak::new());
        EmailAccountsTable::new(tbl_data.clone())
    }

    mod db_row {
        use super::*;
        use test_log::test;

        #[test(tokio::test)]
        async fn new_defaults() {
            let row = DbDataRow::new(123, "foo".to_string());
            assert_eq!(row.rowid, 123);
            assert_eq!(row.user, None);
            assert_eq!(row.email, "foo");
            assert_eq!(row.token, None);
            assert_eq!(row.token_creation, None);
            assert_eq!(row.token_consumption, None);
        }

        #[test(tokio::test)]
        async fn load_store() {
            let pool = get_pool().await;
            let dt1: DateTime<Utc> = DateTime::parse_from_rfc3339("1001-01-01T01:01:01.1111+01:00").unwrap().into();
            let dt2: DateTime<Utc> = DateTime::parse_from_rfc3339("2002-02-02T02:02:02.2222+02:00").unwrap().into();
            let token = "as3245lkds".to_string();

            // generate some test data
            let mut query = sqlx::query("INSERT INTO users (rowid,name) VALUES (123,'Foo');");
            query.execute(&pool).await.unwrap();

            // store
            let mut row = DbDataRow::new(0, "a.b@c.de".to_string());
            row.user = Some(123);
            row.token = Some(token.clone());
            row.token_creation = Some(dt1);
            row.token_consumption = Some(dt2);
            row.store(&pool).await.unwrap();  // check INSERT
            assert_eq!(row.rowid, 1);
            row.store(&pool).await.unwrap();  // check UPDATE
            assert_eq!(row.rowid, 1);

            // load
            let mut row = DbDataRow::new(1, "".to_string());
            row.load(&pool).await.unwrap();
            assert_eq!(row.rowid, 1);
            assert_eq!(row.user, Some(123));
            assert_eq!(row.email, "a.b@c.de".to_string());
            assert_eq!(row.token, Some(token.clone()));
            assert_eq!(row.token_creation, Some(dt1));
            assert_eq!(row.token_consumption, Some(dt2));

            // from email
            assert!(DbDataRow::from_email("wrong.email@nothing.com", &pool).await.is_err());
            let row = DbDataRow::from_email("a.b@c.de", &pool).await.unwrap();
            assert_eq!(row.rowid, 1);
            assert_eq!(row.user, Some(123));
            assert_eq!(row.email, "a.b@c.de".to_string());
            assert_eq!(row.token, Some(token));
            assert_eq!(row.token_creation, Some(dt1));
            assert_eq!(row.token_consumption, Some(dt2));
        }

        #[test(tokio::test)]
        async fn unique_email() {
            let pool = get_pool().await;

            // create first email
            let mut row = DbDataRow::new(0, "a.b@c.de".to_string());
            row.store(&pool).await.unwrap();

            // create same email shall fail
            let mut row = DbDataRow::new(0, "a.b@c.de".to_string());
            assert!(row.store(&pool).await.is_err());

            // create same email shall fail, trimmed
            let mut row = DbDataRow::new(0, " a.b@c.de\n".to_string());
            assert!(row.store(&pool).await.is_err());

            // create same email shall fail, case insensitive
            let mut row = DbDataRow::new(0, "a.B@c.de".to_string());
            assert!(row.store(&pool).await.is_err());
        }

    }

    mod item {
        use super::*;
        use test_log::test;

        async fn create_new_item(pool: &SqlitePool, email: String) -> EmailAccountItem {
            let mut row = DbDataRow::new(0, email);
            row.store(pool).await.unwrap();
            let data = EmailAccountItemData::new(pool, row, Weak::new());
            EmailAccountItem::new(data)
        }

        #[test(tokio::test)]
        async fn new_item() {
            let pool = get_pool().await;
            let item = create_new_item(&pool, "a.b@c.de".to_string()).await;
            assert_eq!(item.id().await, 1);
            assert_eq!(item.email().await, "a.b@c.de".to_string());
            assert_eq!(item.is_verified().await, false);
        }

        #[test(tokio::test)]
        async fn token_verification_process() {
            let pool = get_pool().await;
            let item = create_new_item(&pool, "a.b@c.de".to_string()).await;

            // successfully create and verify token
            assert_eq!(item.is_verified().await, false);
            let token = item.create_token().await.unwrap();
            assert!(token.len() > 20); // ensure to have a secure token
            assert!(item.consume_token(token).await);
            assert_eq!(item.is_verified().await, true);

            // successfully create and verify token (should work a second time)
            let token = item.create_token().await.unwrap();
            assert!(token.len() > 20); // ensure to have a secure token
            assert!(item.consume_token(token).await);
            assert_eq!(item.is_verified().await, true);

            // fail to consume a wrong token
            let mut token = item.create_token().await.unwrap();
            assert!(token.len() > 20); // ensure to have a secure token
            token.push('X'); // manipulate the token
            assert!(!item.consume_token(token.clone()).await);
            assert_eq!(item.is_verified().await, false);

            // fail when token is outdated
            let time_now = Utc::now();
            let time_token_outdated = time_now.clone().checked_add_signed(chrono::TimeDelta::hours(-1)).unwrap();
            // let token = item.create_token().await.unwrap();  // not generating new token, because last token is still active
            let id = item.id().await;
            let mut row = DbDataRow::new(id, "".to_string());
            row.load(&pool).await.unwrap();
            row.token_creation = Some(time_token_outdated);
            row.store(&pool).await.unwrap();
            let item_data = EmailAccountItemData::new(&pool, row, Weak::new());
            let item = EmailAccountItem::new(item_data.clone());
            assert!(!item.consume_token(token).await);
            assert_eq!(item.is_verified().await, false);
        }
    }

    mod table {
        use super::*;
        use test_log::test;

        async fn create_table() -> EmailAccountsTable {
            let pool = get_pool().await;
            let table_data = EmailAccountsTableData::new(pool, Weak::new());
            EmailAccountsTable::new(table_data)
        }

        #[test(tokio::test)]
        async fn new() {
            create_table().await;
        }

        #[test(tokio::test)]
        async fn create_account() {
            let tbl = create_table().await;

            // with valid email
            let item = tbl.create_account("a.b@c.de".to_string()).await;
            assert!(item.is_some());

            // with invalid email
            let item = tbl.create_account("@foo.com".to_string()).await;
            assert!(item.is_none());

            // deny creating new account with same email
            let item = tbl.create_account("a.b@c.de".to_string()).await;
            assert!(item.is_none());

            // with valid email, that failed once
            let item = tbl.create_account("Thomas.Weinhold@stratoinos.de".to_string()).await;
            assert!(item.is_some());
        }

        #[test(tokio::test)]
        async fn by_id_by_email() {
            let tbl = create_table().await;

            // create two emails
            let item = tbl.create_account("a.b@c.de".to_string()).await.unwrap();
            let token = item.create_token().await.unwrap();
            assert!(item.consume_token(token).await);
            tbl.create_account("siegmund.jaehn@space.de".to_string()).await.unwrap();

            // by id
            assert_eq!(tbl.item_by_id(1).await.unwrap().email().await,
                       "a.b@c.de".to_string());
            assert_eq!(tbl.item_by_id(2).await.unwrap().email().await,
                       "siegmund.jaehn@space.de".to_string());

            // by email, verified
            assert!(tbl.item_by_email("a.b@c.de").await.is_some());
            assert!(tbl.item_by_email("siegmund.jaehn@space.de").await.is_none());

            // by email, unverified
            assert!(tbl.item_by_email_ignore_verification("a.b@c.de").await.is_some());
            assert!(tbl.item_by_email_ignore_verification("siegmund.jaehn@space.de").await.is_some());

        }
    }
}
