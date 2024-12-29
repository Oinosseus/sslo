
/// This is the central defined name of the table in this module,
/// used to allow copy&paste of the code for other tables.
macro_rules! tablename {
    () => { "users" };
}

use std::collections::HashMap;
use std::sync::{Arc, RwLock, Weak};
use crate::user_grade::{Promotion, PromotionAuthority};
use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};
use sslo_lib::db::PoolPassing;

/// Data structure that is used for database interaction (only module internal use)
#[derive(sqlx::FromRow, Clone)]
struct DataRow {
    rowid: i64,
    name: String,
    promotion_authority: PromotionAuthority,
    promotion: Promotion,
    last_lap: Option<DateTime<Utc>>,
    email: Option<String>,
    email_token: Option<String>,
    email_token_creation: Option<DateTime<Utc>>,
    email_token_consumption: Option<DateTime<Utc>>,
    password: Option<String>,
    password_last_usage: Option<DateTime<Utc>>,
    password_last_useragent: Option<String>,
}

impl DataRow {

    /// Create a new (empty/default) data row
    pub(super) fn new(rowid: i64) -> Self {
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

    /// Internal service function to retrieve an SqlitePool
    fn get_pool(pool_ref: Weak<dyn PoolPassing>) -> Option<SqlitePool> {

        // get pool-passer reference
        let pool_passer = match pool_ref.upgrade() {
            Some(x) => {x},
            None => {
                log::error!("Dead reference to PoolPassing object!");
                return None;
            }
        };

        // get pool
        let pool = match pool_passer.pool() {
            Some(x) => x,
            None => {
                log::error!("No pool available!");
                return None;
            }
        };

        Some(pool)
    }


    /// Read the data from the database
    /// This consumes a Row object and returns a new row object on success
    pub(super) async fn load(mut self, pool_passer: Weak<dyn PoolPassing>) -> Option<Self> {
        let pool = Self::get_pool(pool_passer)?;

        match sqlx::query_as(concat!("SELECT rowid,* FROM ", tablename!(), " WHERE rowid = $1 LIMIT 2;"))
            .bind(self.rowid)
            .fetch_one(&pool)
            .await {
            Ok(row) => {
                return Some(row);
            },
            Err(sqlx::Error::RowNotFound) => {
                return None;
            },
            Err(e) => {
                log::error!("Failed to query database: {}", e);
                return None;
            }
        };

    }

    /// Write the data into the database
    /// When rowid is unequal to '0', an UPDATE is executed,
    /// When rowid is zero, an insert is executed and rowid is updated
    /// When INSERT fails, rowid will stay at zero
    pub(super) async fn store(mut self, pool_passer: Weak<dyn PoolPassing>) -> Option<Self> {
        let pool = Self::get_pool(pool_passer)?;

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

        // execute query
        if self.rowid == 0 {
            match query.fetch_one(&pool).await {
                Ok(res) => self.rowid = res.get(0),
                Err(e) => {
                    log::error!("Failed to insert into db: {}", e);
                    return None;
                }
            }

        } else {
            query = query.bind(self.rowid);
            match query.execute(&pool).await {
                Ok(_) => {},
                Err(e) => {
                    log::error!("Failed to update rowid={}: {}", self.rowid, e);
                    return None;
                }
            }
        }

        Some(self)
    }
}


/// This abstracts access to single items within the table
pub struct Item {
    pool_ref_2parent: Weak<dyn PoolPassing>,
    row: DataRow,
}

impl Item {

    pub fn id(&self) -> i64 { self.row.rowid}

    /// Set up an object from a data row (assumed to be retrieved from database)
    pub(super) fn from_row(pool_ref: Weak<dyn PoolPassing>, row: DataRow) -> Arc<Self> {
        Arc::new(Self { pool_ref_2parent: pool_ref, row } )
    }

    // /// Retrieve data row from database
    // pub(super) async fn from_id(pool_passer: Arc<dyn PoolPassing>, id: i64) -> Option<Self> {
    //
    //     // get pool
    //     let pool = match pool_passer.pool() {
    //         Some(pool) => pool,
    //         None => {
    //             #[cfg(debug_assertions)]
    //             panic!("No pool!");
    //             log::error!("No pool!");
    //             return None;
    //         }
    //     };
    //
    //     // request row from database
    //     let mut data_row = match DataRow::new(id).load(&pool).await {
    //         Some(dr) => dr,
    //         None => return None,
    //     };
    //
    //     // return
    //     Some(Self{
    //         pool_ref_2parent: Arc::downgrade(&pool_passer),
    //         data_row,
    //     })
    // }

    // /// Retrieve data row from database
    // pub(super) async fn from_db_by_email(pool: &SqlitePool, email: &str) -> Option<Self> {
    //
    //     // query
    //     let mut rows: Vec<Row> = match sqlx::query_as(concat!("SELECT rowid,* FROM ", tablename!(), " WHERE email LIKE $1 LIMIT 2;"))
    //         .bind(email)
    //         .fetch_all(pool)
    //         .await {
    //         Ok(r) => r,
    //         Err(e) => {
    //             log::error!("Failed to query database: {}", e);
    //             return None;
    //         }
    //     };
    //
    //     // ambiguity check
    //     #[cfg(debug_assertions)]
    //     if rows.len() > 1 {
    //         log::error!("Ambiguous email={}", email);
    //         return None;
    //     }
    //
    //     // return item
    //     rows.pop()
    // }

    /// datetime of last driven lap
    pub fn last_lap(&self) -> Option<DateTime<Utc>> {
        self.row.last_lap
    }

    /// name of the user
    pub fn name_ref(&self) -> &str {
        &self.row.name
    }

    pub fn promotion(&self) -> Promotion {
        self.row.promotion.clone()
    }

    pub fn promotion_authority(&self) -> PromotionAuthority {
        self.row.promotion_authority.clone()
    }

    /// Test if the user has a password set
    pub fn has_password(&self) -> bool {
        self.row.password.is_some()
    }
}


/// Main interface to access the table
pub struct Table {
    pool_ref_2me: Weak<dyn PoolPassing>,
    pool_ref_2parent: Weak<dyn PoolPassing>,
    item_cache: RwLock<HashMap<i64, Arc<Item>>>,
}

impl Table {

    /// Create a new table object
    pub(super) fn new(pool_ref: Weak<dyn PoolPassing>) -> Arc<Self> {
        Arc::new_cyclic(|me: &Weak<Self>| {
            Self {
                pool_ref_2me: me.clone(),
                pool_ref_2parent: pool_ref,
                item_cache: RwLock::new(HashMap::new()),
            }
        })
    }


    // /// Get an item
    // /// This first tries to load the item from cache,
    // /// and secondly load it from the database.
    // pub async fn item_by_id(&self, id: i64) -> Option<Arc<Item>> {
    //
    //     // cache hit
    //     match self.item_cache.read() {
    //         Err(e) => {
    //             log::error!("Failed to read item cache: {}", e);
    //             return None;
    //         },
    //         Ok(cache) => {
    //             if let Some(item) = cache.get(&id) {
    //                 return Some(item.clone());
    //             }
    //         }
    //     }
    //
    //     // try loading from DB if not found in cache
    //     match self.item_cache.write() {
    //         Err(e) => {
    //             log::error!("Failed to write-lock cache: {}", e);
    //             return None;
    //         },
    //         Ok(mut cache) => {
    //
    //             // get pool
    //             let pool = match self.pool() {
    //                 Some(pool) => pool,
    //                 None => {
    //                     log::error!("No pool!");
    //                     return None;
    //                 }
    //             };
    //
    //             // request item from db
    //             let row = Row::from_db_by_id(&pool, id).await.or_else(||{
    //                 log::warn!("No db item with id={} found!", id);
    //                 None
    //             })?;
    //             let item = Item::from_row(self.pool_ref_2me.clone(), row);
    //
    //             // update cache
    //             cache.insert(item.id(), item.clone());
    //
    //             // return
    //             Some(item)
    //         }
    //     }
    // }


    // /// Search the database for an email and then return the item
    // /// The search is case-insensitive
    // pub async fn item_by_email(&self, email: &str) -> Option<Arc<Item>> {
    //
    //     // get pool
    //     let pool = match self.pool() {
    //         Some(pool) => pool,
    //         None => {
    //             log::error!("No pool!");
    //             return None;
    //         }
    //     };
    //
    //     // find item in database
    //     let row = Row::from_db_by_email(&pool, email).await.or_else(||{
    //         log::warn!("No item with email={} found!", email);
    //         None
    //     })?;
    //     self.item_by_id(row.rowid).await
    // }
}

impl PoolPassing for Table {
    fn pool(&self) -> Option<SqlitePool> {
        self.pool_ref_2parent.upgrade()?.pool()
    }
}


#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use sqlx::SqlitePool;
    use super::*;
    use sslo_lib::db::PoolPassing;
    use std::sync::{Arc, Weak};
    use env_logger;
    use test_log::test;

    struct TestPoolPasser {
        pub pool: SqlitePool,
        pub pool_ref_2me: Weak<dyn PoolPassing>,
    }
    impl TestPoolPasser {
        fn new() -> Arc<Self> {
            let sqlite_opts = SqliteConnectOptions::from_str(":memory:").unwrap();
            let pool = SqlitePoolOptions::new()
                .min_connections(1)
                .max_connections(1)  // default is 10
                .idle_timeout(None)
                .max_lifetime(None)
                .connect_lazy_with(sqlite_opts);
            Arc::new_cyclic(|me: &Weak<Self>| {
                Self{pool, pool_ref_2me: me.clone()}
            })
        }
        async fn init(&self) {
            sqlx::migrate!("../rsc/db_migrations/league_members").run(&self.pool).await.unwrap();
        }
        async fn load_default(&self) {
            sqlx::query(concat!("INSERT INTO ", tablename!(), " (name, email) VALUES ($1, $2);"))
                .bind("username")
                .bind("user@email.tld")
                .execute(&self.pool)
                .await.unwrap();
        }
    }
    impl PoolPassing for TestPoolPasser {
        fn pool(&self) -> Option<SqlitePool> {Some(self.pool.clone())}
    }

    #[test(tokio::test)]
    async fn test_row_new_defaults() {
        let pool_passer = TestPoolPasser::new();
        pool_passer.init().await;
        let pool_ref = &pool_passer.pool().unwrap();

        // test defaults of Row::new()
        let mut row = DataRow::new(33);
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
    async fn test_row_load_store() {
        let pool_passer = TestPoolPasser::new();
        pool_passer.init().await;
        let pp_ref = pool_passer.pool_ref_2me.clone();

        // define some UTC times
        let dt1: DateTime<Utc> = DateTime::parse_from_rfc3339("1001-01-01T01:01:01.1111+01:00").unwrap().into();
        let dt2: DateTime<Utc> = DateTime::parse_from_rfc3339("2002-02-02T02:02:02.2222+02:00").unwrap().into();
        let dt3: DateTime<Utc> = DateTime::parse_from_rfc3339("3003-03-03T03:03:03.3333+03:00").unwrap().into();
        let dt4: DateTime<Utc> = DateTime::parse_from_rfc3339("4004-04-04T04:04:04.4444+04:00").unwrap().into();

        // store (insert)
        let mut row = DataRow::new(0);
        row.name = "RowName".to_string();
        row.promotion_authority = PromotionAuthority::Chief;
        row.promotion = Promotion::Commissar;
        row.last_lap =  Some(dt1.clone());
        row.email = Some("user@email.tld".into());
        row.email_token = Some("IAmAnEmailToken".to_string());
        row.email_token_creation = Some(dt2.clone());
        row.email_token_consumption = Some(dt3.clone());
        row.password = Some("IAmThePassword".to_string());
        row.password_last_usage = Some(dt4.clone());
        row.password_last_useragent = Some("IAmTheUserAgent".to_string());
        row.store(pp_ref.clone()).await;

        // load
        let row = DataRow::new(1).load(pp_ref.clone()).await.unwrap();
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
        let mut row = DataRow::new(1);
        row.name = "RowNameNew".to_string();
        row.promotion_authority = PromotionAuthority::Executing;
        row.promotion = Promotion::Admin;
        row.last_lap =  Some(dt2.clone());
        row.email = Some("a.b@c.de".into());
        row.email_token = Some("IAmAnEmailTokenNew".to_string());
        row.email_token_creation = Some(dt3.clone());
        row.email_token_consumption = Some(dt4.clone());
        row.password = Some("IAmThePasswordNew".to_string());
        row.password_last_usage = Some(dt1.clone());
        row.password_last_useragent = Some("IAmTheUserAgentNew".to_string());
        row.store(pp_ref.clone()).await;

        // load
        let row = DataRow::new(1).load(pp_ref.clone()).await.unwrap();
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

        // create dead pool-passer reference
        let pp_ref_dead:Weak<dyn PoolPassing>;
        {
            let pool_passer = TestPoolPasser::new();
            pool_passer.init().await;
            pp_ref_dead = pool_passer.pool_ref_2me.clone();
        }

        // check failed load/store
        let mut row = DataRow::new(1).store(pp_ref_dead.clone()).await;
        assert!(row.is_none());
        let row = DataRow::new(1).load(pp_ref_dead.clone()).await;
        assert!(row.is_none());
    }

    #[test(tokio::test)]
    async fn test_item_new() {
        let pool_passer = TestPoolPasser::new();
        pool_passer.init().await;
        let pp_ref = pool_passer.pool_ref_2me.clone();

        let row = DataRow::new(0);
        let item = Item::from_row(pp_ref.clone(), row);
        assert_eq!(item.id(), 0);
    }
}