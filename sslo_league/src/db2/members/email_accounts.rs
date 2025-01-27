use std::collections::HashMap;
use std::sync::{Arc, Weak};
use chrono::{DateTime, Utc};
use sqlx::{FromRow, Sqlite, SqlitePool};
use tokio::sync::RwLock;
use sslo_lib::error::SsloError;
use crate::db2::members::MembersDbData;
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
                  token_creation,\
                  token_consumption)\
                  VALUES ($1, $2, $3, $4, $5) RETURNING rowid;"))
            },
            _ => {
                sqlx::query(concat!("UPDATE ", tablename!(), " SET \
                                   user=$1,\
                                   email=$2,\
                                   token=$3,\
                                   token_creation=$4,\
                                   token_consumption=$5,\
                                   WHERE rowid=$6;"))
            }
        };

        // bind values
        query = query.bind(&self.user)
            .bind(&self.email)
            .bind(&self.token)
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

    pub async fn id(&self) -> i64 {
        self.0.read().await.row.rowid
    }

    pub async fn user(&self) -> Option<UserItem> {
        todo!();
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
                log::error!("Token creation/consumption time mismatch for rowid={}, email='{:?}', consumption='{}'",
                            item_data.row.rowid, item_data.row.email, some_token_consumption);
                false
            },
            Some(some_token_consumption) => match token_creation {
                Some(some_token_creation) if some_token_creation > some_token_consumption => {
                    log::error!("Token creation/consumption time mismatch for rowid={}, email='{:?}', creation='{}', consumption='{}'",
                        item_data.row.rowid,
                        item_data.row.email,
                        some_token_creation,
                        some_token_consumption,
                    );
                    false
                },
                Some(_) => true,
                None => {
                    log::error!("Token consumed, but never created for rowid={}, email='{:?}', consumption='{}'",
                        item_data.row.rowid,
                        item_data.row.email,
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
    pub async fn create_token(&self) -> Option<String> {
        todo!();
    }
}

struct EmailAccountsTableData {
    pool: SqlitePool,
    item_cache_by_id: HashMap<i64, Arc<RwLock<EmailAccountItemData>>>,
    item_cache_by_email: HashMap<String, Arc<RwLock<EmailAccountItemData>>>,
}

impl EmailAccountsTableData {
    fn new(pool: SqlitePool) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new( Self {
            pool,
            item_cache_by_id: HashMap::new(),
            item_cache_by_email: HashMap::new(),
        }))
    }
}

pub struct EmailAccountsTable(
    Arc<RwLock<EmailAccountsTableData>>
);

impl EmailAccountsTable {

    fn new(data: Arc<RwLock<EmailAccountsTableData>>) -> Self {
        Self(data)
    }

    /// creates a new email account
    pub async fn create_account(&self, email: String) -> Option<EmailAccountItem> {
        let mut tbl_data = self.0.write().await;

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
        let item_data = EmailAccountItemData::new(&tbl_data.pool, row, Weak::new());
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
            let item_data = EmailAccountItemData::new(&tbl_data.pool, row, Weak::new());
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
            debug_assert_eq!(row.email, email);

            // create item
            let id = row.rowid;
            let item_data = EmailAccountItemData::new(&tbl_data.pool, row, Weak::new());
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
        let tbl_data = EmailAccountsTableData::new(pool);
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

            // store
            let mut row = DbDataRow::new(0, "a.b@c.de".to_string());
            row.user = Some(123);
            row.token = Some(token.clone());
            row.token_creation = Some(dt1);
            row.token_consumption = Some(dt2);
            row.store(&pool).await.unwrap();
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

        mod item {
            use super::*;
            use test_log::test;

            async fn create_new_item(pool: &SqlitePool, email: String) -> EmailAccountItem {
                let row = DbDataRow::new(0, email);
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
        }
    }
}