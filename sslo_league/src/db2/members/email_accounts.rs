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
    user: Option<String>,
    email: String,
    token: Option<String>,
    token_creation: Option<DateTime<Utc>>,
    token_consumption: Option<DateTime<Utc>>,
}

impl DbDataRow {
    fn new(rowid: i64, email: String) -> Self {
        debug_assert!(rowid > 0);
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

        // define query
        let mut query = match self.rowid {
            0 => {
                sqlx::query(concat!("INSERT INTO ", tablename!(),
                "(user,\
                  email,\
                  token,\
                  token_creation,\
                  token_consumption,\
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

    }

    pub async fn email(&self) -> String {
        self.0.read().await.row.email.clone()
    }

    /// Returns true, if the email token has been verified
    pub async fn is_verified(&self) -> bool {

    }

    /// Creates a new token, that shall be sent via email
    /// The token is not created, if an existing token is still pending.
    /// A token is pending until verified or until a timeout has passed.
    pub async fn create_token(&self) -> Option<String> {

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

    pub fn create_account() -> Option<EmailAccountItem> {

    }

    /// Get an item by email address (also returns unverified emails)
    pub async fn item_by_email_ignore_verification(&self, email: &str) -> Option<EmailAccountItem> {

    }

    /// Get an item by email address (only verified emails)
    pub async fn item_by_email(&self, email: &str) -> Option<EmailAccountItem> {

    }
}