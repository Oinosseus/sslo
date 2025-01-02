use chrono::{DateTime, Utc};
use sqlx::{Sqlite, SqlitePool};
use sslo_lib::db::DatabaseError;
use sslo_lib::token;
use super::tablename;

/// Data structure that is used for database interaction (only module internal use)
#[derive(sqlx::FromRow, Clone)]
pub(super) struct ItemDbRow {
    pub(super) rowid: i64,
    pub(super) user: i64,
    pub(super) token: String,
    pub(super) creation: DateTime<Utc>,
    pub(super) last_useragent: Option<String>,
    pub(super) last_usage: Option<DateTime<Utc>>,
}

impl ItemDbRow {

    /// Create a new (empty/default) data row
    pub(super) fn new(rowid: i64) -> Self {
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

    /// Read the data from the database
    /// This consumes a Row object and returns a new row object on success
    pub(super) async fn load(self: &mut Self, pool: &SqlitePool) -> Result<(), DatabaseError> {
        return match sqlx::query_as::<Sqlite, ItemDbRow>(concat!("SELECT rowid,* FROM ", tablename!(), " WHERE rowid = $1 LIMIT 2;"))
            .bind(self.rowid)
            .fetch_one(pool)
            .await {
            Ok(row) => {
                row.clone_into(self);
                Ok(())
            },
            Err(sqlx::Error::RowNotFound) => {
                Err(DatabaseError::RowidNotFound(tablename!(), self.rowid))
            },
            Err(e) => {
                Err(DatabaseError::SqlxLowLevelError(e))
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
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use sqlx::SqlitePool;
    use super::*;
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
        let row = ItemDbRow::new(33);
        assert_eq!(row.rowid, 33);
        assert_eq!(row.user, 0);
        assert_eq!(row.token, String::new());
        assert_eq!(row.last_usage, None);
        assert_eq!(row.last_useragent, None);
    }

    /// Testing load and store (insert+update)
    #[test(tokio::test)]
    async fn load_store() {
        let pool = get_pool().await;

        // define some UTC times
        let dt1: DateTime<Utc> = DateTime::parse_from_rfc3339("1001-01-01T01:01:01.1111+01:00").unwrap().into();
        let dt2: DateTime<Utc> = DateTime::parse_from_rfc3339("2002-02-02T02:02:02.2222+02:00").unwrap().into();
        let dt3: DateTime<Utc> = DateTime::parse_from_rfc3339("3003-03-03T03:03:03.3333+03:00").unwrap().into();

        // store (insert)
        let mut row = ItemDbRow::new(0);
        row.user = 44;
        row.token = "MyInsecureTestToken".to_string();
        row.creation = dt1;
        row.last_usage = Some(dt2);
        row.last_useragent = Some("unit test".to_string());
        row.store(&pool).await.unwrap();

        // load
        let mut row = ItemDbRow::new(1);
        row.load(&pool).await.unwrap();
        assert_eq!(row.rowid, 1);
        assert_eq!(row.user, 44);
        assert_eq!(row.token, "MyInsecureTestToken".to_string());
        assert_eq!(row.creation, dt1.clone());
        assert_eq!(row.last_usage, Some(dt2.clone()));
        assert_eq!(row.last_useragent, Some("unit test".to_string()));

        // store (update)
        let mut row = ItemDbRow::new(1);
        row.user = 46;
        row.token = "MyNewInsecureTestToken".to_string();
        row.creation = dt2;
        row.last_usage = Some(dt3);
        row.last_useragent = Some("new unit test".to_string());
        row.store(&pool).await.unwrap();

        // load
        let mut row = ItemDbRow::new(1);
        row.load(&pool).await.unwrap();
        assert_eq!(row.rowid, 1);
        assert_eq!(row.user, 46);
        assert_eq!(row.token, "MyNewInsecureTestToken".to_string());
        assert_eq!(row.creation, dt2.clone());
        assert_eq!(row.last_usage, Some(dt3.clone()));
        assert_eq!(row.last_useragent, Some("new unit test".to_string()));
    }
}
