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
            log::error!("Ambiguous rowid for db_obsolete.members.steam_users.rowid={}", rowid);
            return None;
        }

        // return
        if let Some(row) = rows.pop() { Some(Self {row, pool}) }
        else { None }
    }


    pub async fn new_or_from_steam_id(pool: SqlitePool, steam_id: &str) -> Option<Self> {

        // find steam id
        let mut rows: Vec<DbRow> = match sqlx::query_as("SELECT rowid,* FROM steam_users WHERE steam_id = $1 LIMIT 2;")
            .bind(steam_id)
            .fetch_all(&pool)
            .await {
            Ok(r) => r,
            Err(e) => {
                log::error!("Failed to query database: {}", e);
                return None;
            }
        };

        // return when unique ID is found
        if rows.len() > 1 {
            log::error!("Ambiguous steam_id for db_obsolete.members.steam_users.steam_id={}", steam_id);
            return None;
        } else if let Some(row) = rows.pop() {
            return Some(Self{ row, pool});
        }

        // insert new
        let row: DbRow = match sqlx::query_as("INSERT INTO steam_users (steam_id) VALUES ($1) RETURNING rowid,*;")
            .bind(steam_id)
            .fetch_one(&pool)
            .await {
            Ok(x) => x,
            Err(e) => {
                log::error!("Failed to insert into db_obsolete.members.steam_users: {}", e);
                return None;
            }
        };

        // return
        Some(Self{ row, pool})
    }
}