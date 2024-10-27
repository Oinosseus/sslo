use std::error::Error;
use std::fs::Permissions;
use sqlx::SqlitePool;

#[derive(sqlx::FromRow)]
pub struct Item {
    pub rowid: i64,
    pub name: String,
    pub permission: i64,
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
}