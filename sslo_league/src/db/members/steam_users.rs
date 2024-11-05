use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

#[derive(sqlx::FromRow)]
struct DbRow {
    pub rowid: i64,
    pub steam_id: String,
    pub creation: DateTime<Utc>,
    pub user: Option<i64>,
    pub last_login_timestamp: Option<DateTime<Utc>>,
    pub last_login_useragent: Option<String>,
}

pub struct SteamUser {
    row: DbRow,
    pool: SqlitePool,
}

impl SteamUser {
    pub async fn from_id(pool: SqlitePool, rowid: i64) -> Option<Self> {

        // query
        let mut rows = match sqlx::query_as("SELECT rowid,* FROM steam_users WHERE rowid = $1 LIMIT 2;")
            .bind(rowid)
            .fetch_all(&pool)
            .await {
            Ok(r) => r,
            Err(e) => {
                log::error!("Failed to query database: {}", e);
                return None;
            }
        };

        // ambiguity check
        if rows.len() > 1 {
            log::error!("Ambiguous rowid for db.members.steam_users.rowid={}", rowid);
            return None;
        }

        // return
        if let Some(row) = rows.pop() { Some(Self {row, pool}) }
        else { None }
    }
}