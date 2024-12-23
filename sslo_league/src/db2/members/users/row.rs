use chrono::{DateTime, Utc};
use sqlx::{Error, SqlitePool};
use crate::user_grade;
use crate::user_grade::{Promotion, PromotionAuthority};

#[derive(sqlx::FromRow, Clone)]
pub(super) struct Row {
    pub rowid: i64,
    pub name: String,
    pub promotion_authority: user_grade::PromotionAuthority,
    pub promotion: user_grade::Promotion,
    pub last_lap: Option<DateTime<Utc>>,
    pub email: Option<String>,
    pub email_token: Option<String>,
    pub email_token_creation: Option<DateTime<Utc>>,
    pub email_token_consumption: Option<DateTime<Utc>>,
    pub password: Option<String>,
    pub password_last_usage: Option<DateTime<Utc>>,
    pub password_last_useragent: Option<String>,
}

impl Row {

    /// Create a new (empty/default) data row
    pub(super) fn new() -> Self {
        Self {
            rowid: 0,
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

    /// Retrieve data row from database
    pub(super) async fn from_db_by_id(pool: &SqlitePool, id: i64) -> Option<Self> {

        // query
        let mut rows: Vec<Self> = match sqlx::query_as(concat!("SELECT rowid,* FROM ", tablename!(), " WHERE rowid = $1 LIMIT 2;"))
            .bind(id)
            .fetch_all(pool)
            .await {
            Ok(r) => r,
            Err(e) => {
                log::error!("Failed to query database: {}", e);
                return None;
            }
        };

        // ambiguity check
        #[cfg(debug_assertions)]
        if rows.len() > 1 {
            log::error!("Ambiguous rowid={}", id);
            return None;
        }

        // return item
        rows.pop()
    }

    /// Retrieve data row from database
    pub(super) async fn from_db_by_email(pool: &SqlitePool, email: &str) -> Option<Self> {

        // query
        let mut rows: Vec<Self> = match sqlx::query_as(concat!("SELECT rowid,* FROM ", tablename!(), " WHERE email LIKE $1 LIMIT 2;"))
            .bind(email)
            .fetch_all(pool)
            .await {
            Ok(r) => r,
            Err(e) => {
                log::error!("Failed to query database: {}", e);
                return None;
            }
        };

        // ambiguity check
        #[cfg(debug_assertions)]
        if rows.len() > 1 {
            log::error!("Ambiguous email={}", email);
            return None;
        }

        // return item
        rows.pop()
    }

    /// Update a data row from the database
    /// When not existing in database, nothing is updated
    pub(super) async fn load(&mut self, pool: &SqlitePool) {

        // sanity check
        if self.rowid < 1 {
            log::error!("Skip loading invalid rowid={}!", self.rowid);
            return;
        }

        // get from DB
        if let Some(mut db_row) = Self::from_db_by_id(pool, self.rowid).await {
            db_row.clone_into(self);
        }
    }


    /// Updates the data into the database
    /// When rowid is greater or equal to 1, an existing item in db it is updated.
    /// When rowid is zero (or negative), a new item is inserted into the database (and returned)
    pub(super) async fn store(&mut self, pool: &SqlitePool) {

        // update
        if (self.rowid > 0) {
            match sqlx::query("UPDATE users SET name=$1,\
                                                    promotion_authority=$2,\
                                                    promotion=$3,\
                                                    last_lap=$4,\
                                                    email=$5,\
                                                    email_token=$6,\
                                                    email_token_creation=$7,\
                                                    email_token_consumption=$8,\
                                                    password=$9,\
                                                    password_last_usage=$10,\
                                                    password_last_useragent=$11,\
                                                    WHERE rowid=$12;")
                .bind(&self.name)
                .bind(&self.promotion_authority)
                .bind(&self.promotion)
                .bind(&self.last_lap)
                .bind(&self.email)
                .bind(&self.email_token)
                .bind(&self.email_token_creation)
                .bind(&self.email_token_consumption)
                .bind(&self.password)
                .bind(&self.password_last_usage)
                .bind(&self.password_last_useragent)
                .bind(&self.rowid)
                .execute(pool)
                .await {
                    Ok(_) => {},
                    Err(e) => {
                        log::error!("Failed to update db.members.users.rowid={}", self.rowid);
                    }
            }


        // insert
        } else {
            let res: Result<i64, Error> = sqlx::query_scalar(concat!("INSERT INTO ", tablename!(),
                                          "(name, promotion_authority, promotion, last_lap, email, email_token, email_token_creation, email_token_consumption, password, password_last_usage, password_last_useragent) ",
                                          "VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING rowid;"))
                .bind(&self.name)
                .bind(&self.promotion_authority)
                .bind(&self.promotion)
                .bind(&self.last_lap)
                .bind(&self.email)
                .bind(&self.email_token)
                .bind(&self.email_token_creation)
                .bind(&self.email_token_consumption)
                .bind(&self.password)
                .bind(&self.password_last_usage)
                .bind(&self.password_last_useragent)
                .fetch_one(pool)
                .await;

            match res {
                    Ok(id) => self.rowid = id,
                    Err(e) => {
                        log::error!("Failed to insert into db: {}", e);
                        self.rowid = 0;
                    }
            }
        }
    }
}
