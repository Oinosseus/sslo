use std::error::Error;
use sqlx::SqlitePool;


#[derive(sqlx::FromRow)]
pub struct Item {
    pub rowid: i64,
    pub name: String,
    pub promotion_authority: crate::user_grade::PromotionAuthority,
    pub promotion: crate::user_grade::Promotion,
    pub last_lap: Option<chrono::DateTime<chrono::Utc>>,
}


#[derive(Clone)]
pub struct Table {
    db_pool: SqlitePool
}

impl Table {
    pub fn new(db_pool: SqlitePool) -> Self { Self {db_pool}}


    pub async fn from_id(&self, rowid: i64) -> Option<Item> {

        let mut res : Vec<Item> = match sqlx::query_as("SELECT rowid, * FROM users WHERE rowid=$1 LIMIT 2;")
            .bind(rowid)
            .fetch_all(&self.db_pool)
            .await {
            Ok(x) => x,
            Err(e) => {
                log::error!("Failed to request database: {}", e);
                return None;
            },
        };

        // fail on multiple results
        if res.len() > 1 {
            log::error!("Multiple database entries for members.users.rowid={}", rowid);
            return None;
        }

        res.pop()
    }


    /// Insert new entry into users table
    /// Returns rowid on success
    pub async fn new_item(&self, name: &str) -> Result<Item, Box<dyn Error>> {
        let res: Item = sqlx::query_as("INSERT INTO users (name) VALUES ($1) RETURNING rowid, *;")
            .bind(&name)
            .fetch_one(&self.db_pool)
            .await
            .or_else(|e| {
                log::error!("Unable to create new row into db.members.users: {}", e);
                return Err(e);
            })?;
        Ok(res)
    }


    pub async fn set_name(&self, rowid: i64, name: &str) -> Result<Item, Box<dyn Error>> {
        let item = match sqlx::query_as("UPDATE users SET name = $1 WHERE rowid = $2 RETURNING rowid,*;")
            .bind(name)
            .bind(rowid)
            .fetch_one(&self.db_pool)
            .await {
            Ok(i) => i,
            Err(e) => {
                log::error!("Could not update name for db.members.users.rowid={}", rowid);
                return Err(e)?;
            },
        };

        Ok(item)
    }
}