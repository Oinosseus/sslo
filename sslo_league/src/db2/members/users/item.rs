use std::sync::Arc;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use sqlx::SqlitePool;
use super::row::ItemDbRow;
use sslo_lib::db::DatabaseError;
use sslo_lib::token;
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

    /// Returns email address (if correctly confirmed)
    pub async fn email(&self) -> Option<String> {
        let now = Utc::now();
        let item_data = self.0.read().await;

        // ensure email is set
        let email = match item_data.row.email.as_ref() {
            Some(x) => x,
            None => return None,
        };

        // ensure email is verified
        match item_data.row.email_token_consumption.as_ref() {
            Some(t) if t > &now => {
                log::error!("Token creation/consumption time mismatch for rowid={}, email='{}', consumption='{}'",
                            item_data.row.rowid, email, t);
                return None;
            },
            Some(t) => {
                t
            },
            None => {
                log::warn!("hide email, because token not verified for user rowid={}, email={:?}",
                    item_data.row.rowid, item_data.row.email);
                return None;
            }
        };

        return Some(email.clone());
    }

    /// Returns a token, that must be sent to the customer for confirmation
    pub async fn set_email(&mut self, email: String) -> Option<String> {
        // let email = email.to_lowercase();  // convention to store only lower-case
        let mut item_data = self.0.write().await;

        // check for timeout since last token creation
        let time_now = Utc::now();
        let time_token_outdated = time_now.clone()
            .checked_add_signed(chrono::TimeDelta::hours(-1))
            .unwrap();  // subtracting one hour cannot fail, theoretically
        if let Some(token_creation) = item_data.row.email_token_creation {
            if token_creation > time_token_outdated {               // token is still valid
                if item_data.row.email_token_consumption.is_none() {    // token is not used, yet
                    log::warn!("Not generating new email login token for user {}:'{}' because last token is still active.", item_data.row.rowid, email);
                    return None;
                }
            }
        }

        // generate new email_token
        let token = match token::Token::generate(token::TokenType::Strong) {
            Ok(t) => t,
            Err(e) => {
                log::error!("Could not generate new token: {}", e);
                return None;
            }
        };

        // update data
        item_data.row.email = Some(email);
        item_data.row.email_token = Some(token.encrypted);
        item_data.row.email_token_creation = Some(time_now);
        item_data.row.email_token_consumption = None;
        let pool = item_data.pool.clone();
        return match item_data.row.store(&pool).await {
            Ok(_) => Some(token.decrypted),
            Err(e) => {
                log::error!("failed to store new email token for user rowid={} into db: {}", item_data.row.rowid, e);
                None
            }
        };
    }

    pub async fn verify_email(&self, token_decrypted: String) -> bool {
        let mut item_data = self.0.write().await;
        let time_now = Utc::now();
        let time_token_outdated = time_now.clone()
            .checked_add_signed(chrono::TimeDelta::hours(-1))
            .unwrap();  // subtracting one hour cannot fail, theoretically

        // ensure encrypted token is set
        let token_encrypted = match item_data.row.email_token.as_ref() {
            Some(x) => x,
            None => {
                log::warn!("deny email verification because no email token set for user rowid={}; email={:?}",
                    item_data.row.rowid, item_data.row.email);
                return false;
            },
        };

        // ensure token is not already consumed
        if let Some(consumption_time) = item_data.row.email_token_consumption.as_ref() {
            log::warn!("deny email token validation for user rowid={}, email={:?}, because token already consumed at {}",
                        item_data.row.rowid, item_data.row.email, consumption_time);
            return false;
        }

        // ensure creation time is not outdated
        match item_data.row.email_token_creation.as_ref() {
            None => {
                log::error!("deny email verification, because no token-creation time set for user rowid={}; email={:?}",
                    item_data.row.rowid, item_data.row.email);
                return false;
            },
            Some(token_creation) => {
                if token_creation < &time_token_outdated {
                    log::warn!("deny email verification, because token is outdated since {} for user rowid={}; email={:?}",
                                        time_token_outdated, item_data.row.rowid, item_data.row.email);
                    return false;
                }
            },
        }

        // verify token
        if !sslo_lib::token::Token::new(token_decrypted, token_encrypted.clone()).verify() {
            log::warn!("deny email verification because token verification failed for rowid={}, email={:?}",
                item_data.row.rowid, item_data.row.email);
            return false;
        }

        // update email_token_consumption
        item_data.row.email_token = None;  // reset for security
        item_data.row.email_token_consumption = Some(time_now);
        let pool = item_data.pool.clone();
        return match item_data.row.store(&pool).await {
            Ok(_) => true,
            Err(e) => {
                log::error!("failed to store verified email token for rowid={}, email={:?}: {}",
                item_data.row.rowid, item_data.row.email, e);
                false
            }
        }
    }

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
        let item = ItemInterface::new(data);
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
        let item = load_item_from_db(1, &pool).await;
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

    #[test(tokio::test)]
    async fn email() {

        // create item
        let pool = get_pool().await;
        let mut item = create_new_item(&pool.clone()).await;
        assert_eq!(item.email().await, None);

        // set email (expect to be transformed into lower-case)
        let email_token = item.set_email("a.b@c.de".to_string()).await.unwrap();
        assert_eq!(item.email().await, None);
        assert!(item.verify_email(email_token).await);
        assert_eq!(item.email().await, Some("a.b@c.de".to_string()));

        // check if stored into db correctly
        let item = load_item_from_db(item.id().await, &pool).await;
        assert_eq!(item.email().await, Some("a.b@c.de".to_string()));
    }
}