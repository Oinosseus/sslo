use std::error::Error;
use chrono::{DateTime, Utc};
use sqlx::{Sqlite, SqlitePool};
use sslo_lib::db::DatabaseError;
use crate::user_grade::{Promotion, PromotionAuthority};
use super::tablename;

/// Data structure that is used for database interaction (only module internal use)
#[derive(sqlx::FromRow, Clone)]
pub(super) struct ItemDbRow {
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

impl ItemDbRow {

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

    /// Read the data from the database
    /// This consumes a Row object and returns a new row object on success
    pub(super) async fn load(self: &mut Self, pool: &SqlitePool) -> Result<(), DatabaseError> {
        match sqlx::query_as::<Sqlite, ItemDbRow>(concat!("SELECT rowid,* FROM ", tablename!(), " WHERE rowid = $1 LIMIT 2;"))
            .bind(self.rowid)
            .fetch_one(pool)
            .await {
            Ok(row) => {
                row.clone_into(self);
                return Ok(());
            },
            Err(sqlx::Error::RowNotFound) => {
                return Err(DatabaseError::RowidNotFound(tablename!(), self.rowid));
            },
            Err(e) => {
                return Err(DatabaseError::SqlxLowLevelError(e));
            }
        };
    }

    /// Write the data into the database
    /// When rowid is unequal to '0', an UPDATE is executed,
    /// When rowid is zero, an insert is executed and rowid is updated
    /// When INSERT fails, rowid will stay at zero
    pub(super) async fn store(self: &mut Self, pool: &SqlitePool) -> Result<(), DatabaseError> {

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

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use sqlx::SqlitePool;
    use super::*;
    use env_logger;
    use test_log::test;

    async fn get_pool() -> SqlitePool {
        let sqlite_opts = SqliteConnectOptions::from_str(":memory:").unwrap();
        let pool = SqlitePoolOptions::new()
            .min_connections(1)
            .max_connections(1)  // default is 10
            .idle_timeout(None)
            .max_lifetime(None)
            .connect_lazy_with(sqlite_opts);
        sqlx::migrate!("../rsc/db_migrations/league_members").run(&pool).await.unwrap();
        return pool;
    }

    #[test(tokio::test)]
    async fn new_defaults() {
        let mut row = ItemDbRow::new(33);
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
        let pool = get_pool().await;

        // define some UTC times
        let dt1: DateTime<Utc> = DateTime::parse_from_rfc3339("1001-01-01T01:01:01.1111+01:00").unwrap().into();
        let dt2: DateTime<Utc> = DateTime::parse_from_rfc3339("2002-02-02T02:02:02.2222+02:00").unwrap().into();
        let dt3: DateTime<Utc> = DateTime::parse_from_rfc3339("3003-03-03T03:03:03.3333+03:00").unwrap().into();
        let dt4: DateTime<Utc> = DateTime::parse_from_rfc3339("4004-04-04T04:04:04.4444+04:00").unwrap().into();

        // store (insert)
        let mut row = ItemDbRow::new(0);
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
        let mut row = ItemDbRow::new(1);
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
        let mut row = ItemDbRow::new(1);
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
        let mut row = ItemDbRow::new(1);
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
