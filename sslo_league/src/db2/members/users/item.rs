use std::sync::Arc;
use tokio::sync::RwLock;
use sqlx::SqlitePool;
use super::row::ItemDbRow;
use sslo_lib::db::DatabaseError;

/// The actual data of an item that is shared by Arc<RwLock<ItemData>>
pub(super) struct ItemData {
    pool: SqlitePool,
    row: ItemDbRow,
}

impl ItemData {
    pub fn new(pool: &SqlitePool, row: ItemDbRow) -> Arc<RwLock<ItemData>> {
        Arc::new(RwLock::new(Self {
            pool: pool.clone(),
            row,
        }))
    }
}

/// This abstracts data access to shared items
pub struct ItemInterface(Arc<RwLock<ItemData>>);

impl ItemInterface {

    /// Set up an object from shared data (assumed to be retrieved from database)
    pub(super) fn new(item_data: Arc<RwLock<ItemData>>) -> Self {
        Self(item_data)
    }

    pub async fn id(&self) -> i64 {
        self.0.read().await.row.rowid
    }


    pub async fn name(&self) -> String {
        self.0.read().await.row.name.clone()
    }

    pub async fn set_name(self: &mut Self, name: String) -> Result<(), DatabaseError> {
        let mut data = self.0.write().await;
        data.row.name = name;
        let pool = data.pool.clone();
        data.row.store(&pool).await
    }

    // pub fn promotion_authority(&self) -> &PromotionAuthority { &self.row.promotion_authority }
    // pub fn promotion(&self) -> &Promotion { &self.row.promotion }
    // pub async fn set_promotion(&mut self, promotion: Promotion, authority: PromotionAuthority) {
    //     self.row.promotion = promotion;
    //     self.row.promotion_authority = authority;
    //     if self.row.store(self.pool_ref_2parent.clone()).await.is_err() {
    //         log::error!("Failed to set promotion: {}", self.row.rowid);
    //     };
    // }
    //
    // pub fn last_lap(&self) -> Option<DateTime<Utc>> {
    //     self.row.last_lap
    // }
    // pub async fn set_last_lap(self: &mut Self, last_lap: DateTime<Utc>) {
    //     self.row.last_lap = Some(last_lap);
    //     if self.row.store(self.pool_ref_2parent.clone()).await.is_err() {
    //         log::error!("Failed to store last lap: {}", self.row.rowid);
    //     };
    // }
    //
    // /// Returns email address (if correctly confirmed)
    // pub fn email(&self) -> Option<&str> {
    //     let now = Utc::now();
    //     if let Some(email) = self.row.email.as_ref() {
    //         if let Some(token_creation) = self.row.email_token_creation {
    //             if let Some(token_consumption) = self.row.email_token_consumption {
    //                 if token_creation < token_consumption && token_consumption < now {
    //                     return Some(&email);
    //                 } else {
    //                     log::error!("Token creation/consumption time mismatch for rowid={}, email='{}', creation='{}', consumption='{}'",
    //                                 self.row.rowid, email, token_creation, token_consumption);
    //                 }
    //             }
    //         } else {
    //             log::error!("Email token creation not set for rowid={}, email='{}'", self.row.rowid, email);
    //         }
    //     }
    //     return None
    // }

    // /// Returns a token, that must be sent to the customer for confirmation
    // pub async fn set_email(&mut self, email: String) -> Option<String> {
    //     // update email_token
    //     // update email_token_creation
    //     // update email
    //     // return email_token
    //     todo!()
    // }
    //
    // pub async fn verify_email(&self, token: String) -> bool {
    //     // update email_token_consumption
    //     todo!();
    // }
    //
    // /// Consume a cleartext password, and store encrypted
    // pub async fn set_password(&mut self, password: String, user_agent: String) -> bool {
    //     // update password -> ENCRYPTED!!!
    //     // update last_usage
    //     // update last_useragent
    //     todo!()
    // }
    //
    // /// Consumes a cleartext password
    // pub async fn verify_password(&self, password: String, user_agent: String) -> bool {
    //     // update last_usage
    //     // update last_useragent
    //     // verify password -> warn if mismatch
    //     todo!()
    // }
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

    /// test item generation and property access
    #[test(tokio::test)]
    async fn new_item() {
        let pool = get_pool().await;

        // create item
        let row = ItemDbRow::new(0);
        let data = ItemData::new(&pool, row);
        let mut item = ItemInterface::new(data);
        assert_eq!(item.id().await, 0);
        assert_eq!(item.name().await, "");
    }

    #[test(tokio::test)]
    async fn name_and_id() {
        let pool = get_pool().await;

        // create item
        let row = ItemDbRow::new(0);
        let data = ItemData::new(&pool, row);
        let mut item = ItemInterface::new(data);
        assert_eq!(item.id().await, 0);

        // modify item
        assert_eq!(item.name().await, "");
        item.set_name("Ronny".to_string()).await.unwrap();
        assert_eq!(item.id().await, 1);
        assert_eq!(item.name().await, "Ronny");

        // reload
        let mut row = ItemDbRow::new(1);
        row.load(&pool).await.unwrap();
        let data = ItemData::new(&pool, row);
        let mut item = ItemInterface::new(data);
        assert_eq!(item.id().await, 1);
        assert_eq!(item.name().await, "Ronny");
    }
}