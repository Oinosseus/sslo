use std::sync::Arc;
use sqlx::{Sqlite, SqlitePool};
use thiserror::Error;
use tokio::sync::RwLock;

#[derive(Error, Debug)]
pub enum DatabaseError {

    #[error("sqlx database pool cannot be retrieved")]
    PoolUnavailable(),

    #[error("no data in table {0} at rowid={1}")]
    RowidNotFound(&'static str, i64),

    #[error("low level error from sqlx: {0}")]
    SqlxLowLevelError(#[from] sqlx::Error),

    #[error("data-lock failed: {0}")]
    DataLockIssue(String),
}

impl DatabaseError {
    pub fn is_rowid_not_found(&self) -> bool {
        match self {
            DatabaseError::RowidNotFound(_, _) => true,
            _ => false,
        }
    }
}

trait StoreableRow {
    type DatabaseRow;
    fn new(rowid: i64) -> Self::DatabaseRow;
    fn rowid(&self) -> i64;
    async fn load(self: &mut Self, pool: &SqlitePool) -> Result<(), DatabaseError>;
    async fn store(self: &mut Self, pool: &SqlitePool) -> Result<(), DatabaseError>;
}
//
// /// The actual data of an item that is shared by Arc<RwLock<ItemData>>
// pub(super) struct ItemData {
//     pool: SqlitePool,
//     row: ItemDbRow,
// }
//
// impl ItemData {
//     pub fn new(pool: &SqlitePool, row: ItemDbRow) -> Arc<RwLock<ItemData>> {
//         Arc::new(RwLock::new(Self {
//             pool: pool.clone(),
//             row,
//         }))
//     }
// }
//
// /// This abstracts data access to shared items
// trait ItemInterface {
//     type ItemData;
//
//     fn new(item_data: Arc<RwLock<ItemData>>) -> Self {
//         Self(item_data)
//     }
//
//     pub async fn id(&self) -> i64 {
//         self.0.read().await.row.rowid
//     }
//
// }
