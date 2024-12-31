use std::sync::Arc;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use sqlx::SqlitePool;
use super::row::ItemDbRow;
use sslo_lib::db::DatabaseError;
use crate::user_grade::{Promotion, PromotionAuthority};

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

    pub async fn promotion_authority(&self) -> PromotionAuthority { self.0.read().await.row.promotion_authority.clone() }
    pub async fn promotion(&self) -> Promotion { self.0.read().await.row.promotion.clone() }
    pub async fn set_promotion(&mut self, promotion: Promotion, authority: PromotionAuthority) {
        let mut item_data = self.0.write().await;
        item_data.row.promotion = promotion;
        item_data.row.promotion_authority = authority;
        let pool = item_data.pool.clone();
        if let Err(e) = item_data.row.store(&pool).await {
            log::error!("Failed to set promotion: {}", e);
        };
    }

    pub async fn last_lap(&self) -> Option<DateTime<Utc>> { self.0.read().await.row.last_lap }
    pub async fn set_last_lap(self: &mut Self, last_lap: DateTime<Utc>) {
        let mut data = self.0.write().await;
        data.row.last_lap = Some(last_lap);
        let pool = data.pool.clone();
        if let Err(e) = data.row.store(&pool).await {
            log::error!("Failed to set last lap: {}", e);
        };
    }

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

    async fn create_new_item(pool: &SqlitePool) -> ItemInterface {
        let row = ItemDbRow::new(0);
        let data = ItemData::new(pool, row);
        ItemInterface::new(data)
    }

    async fn load_item_from_db(id: i64, pool: &SqlitePool) -> ItemInterface {
        let mut row = ItemDbRow::new(id);
        row.load(pool).await.unwrap();
        let data = ItemData::new(&pool, row);
        ItemInterface::new(data)
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
        let mut item = create_new_item(&pool.clone()).await;
        assert_eq!(item.id().await, 0);

        // modify item
        assert_eq!(item.name().await, "");
        item.set_name("Ronny".to_string()).await.unwrap();
        assert_eq!(item.id().await, 1);
        assert_eq!(item.name().await, "Ronny");

        // check if arrived in database
        let mut item = load_item_from_db(1, &pool).await;
        assert_eq!(item.id().await, 1);
        assert_eq!(item.name().await, "Ronny");
    }

    #[test(tokio::test)]
    async fn promotion() {

        // create item
        let pool = get_pool().await;
        let mut item = create_new_item(&pool.clone()).await;

        // modify item (ne before, eq after)
        assert_ne!(item.promotion().await, Promotion::Marshal);
        assert_ne!(item.promotion_authority().await, PromotionAuthority::Chief);
        item.set_promotion(Promotion::Marshal, PromotionAuthority::Chief).await;
        assert_eq!(item.promotion().await, Promotion::Marshal);
        assert_eq!(item.promotion_authority().await, PromotionAuthority::Chief);

        // check if stored into db correctly
        let item = load_item_from_db(item.id().await, &pool).await;
        assert_eq!(item.promotion().await, Promotion::Marshal);
        assert_eq!(item.promotion_authority().await, PromotionAuthority::Chief);
    }

    #[test(tokio::test)]
    async fn last_lap() {

        // create item
        let pool = get_pool().await;
        let mut item = create_new_item(&pool.clone()).await;

        // prepare test data
        let dt: DateTime<Utc> = DateTime::parse_from_rfc3339("1001-01-01T01:01:01.1111+01:00").unwrap().into();

        // modify item (ne before, eq after)
        assert_eq!(item.last_lap().await, None);
        item.set_last_lap(dt).await;
        assert_eq!(item.last_lap().await, Some(dt));

        // check if stored into db correctly
        let item = load_item_from_db(item.id().await, &pool).await;
        assert_eq!(item.last_lap().await, Some(dt));
    }
}