use std::sync::Arc;
use chrono::{DateTime, Utc};
use rand::RngCore;
use tokio::sync::RwLock;
use sqlx::SqlitePool;
use super::row::UserDbRow;
use sslo_lib::db::DatabaseError;
use sslo_lib::token;
use crate::user_grade::{Promotion, PromotionAuthority};

/// The actual data of an item that is shared by Arc<RwLock<ItemData>>
pub(super) struct UserData {
    pool: SqlitePool,
    row: UserDbRow,
}

impl UserData {
    pub fn new(pool: &SqlitePool, row: UserDbRow) -> Arc<RwLock<UserData>> {
        Arc::new(RwLock::new(Self {
            pool: pool.clone(),
            row,
        }))
    }
}

/// This abstracts data access to shared items
pub struct UserInterface(Arc<RwLock<UserData>>);

impl UserInterface {

    /// Set up an object from shared data (assumed to be retrieved from database)
    pub(super) fn new(item_data: Arc<RwLock<UserData>>) -> Self {
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

    /// Consume a cleartext password, and store encrypted
    /// This checks if the current password is valid
    pub async fn update_password(&mut self, old_password: Option<String>, new_password: Option<String>) -> bool {
        let mut data = self.0.write().await;

        // verify old password
        if let Some(old_password_encrypted) = data.row.password.as_ref() {
            if let Some(old_password_decrypted) = old_password {
                match argon2::verify_encoded(old_password_encrypted, &old_password_decrypted.into_bytes()) {
                    Ok(true) => {},
                    Ok(false) => {
                        log::warn!("deny update password, because invalid old password given for rowid={}", data.row.rowid);
                        return false;
                    },
                    Err(e) => {
                        log::error!("Argon2 failure at verifying passwords: {}", e);
                        return false;
                    }
                }
            } else {
                log::warn!("deny update password, because no old password given for rowid={}", data.row.rowid);
                return false;
            }
        }

        // encrypt new password
        let mut new_password_encrypted: Option<String> = None;
        if let Some(some_new_password) = new_password {
            let mut salt: Vec<u8> = vec![0u8; 64];
            rand::thread_rng().fill_bytes(&mut salt);
            new_password_encrypted = match argon2::hash_encoded(&some_new_password.into_bytes(), &salt, &argon2::Config::default()) {
                Ok(p) => Some(p),
                Err(e) => {
                    log::error!("Argon2 failed to encrypt password: {}", e);
                    return false;
                }
            };
        }

        // update password
        data.row.password = new_password_encrypted;
        data.row.password_last_usage = None;
        data.row.password_last_useragent = None;
        let pool = data.pool.clone();
        if let Err(e) = data.row.store(&pool).await {
            log::error!("failed to store updated password for rowid={}: {}", data.row.rowid, e);
            return false;
        }

        log::info!("password updated for rowid={}", data.row.rowid);
        return true;
    }

    /// Consumes a cleartext password
    pub async fn verify_password(&self, password: String, user_agent: String) -> bool {

        {   // separate scope with read-lock for quick return at verification fail
            let data = self.0.read().await;
            if let Some(old_password_encrypted) = data.row.password.as_ref() {
                match argon2::verify_encoded(old_password_encrypted, &password.into_bytes()) {
                    Ok(true) => {}
                    Ok(false) => {
                        return false;
                    }
                    Err(e) => {
                        log::error!("Argon2 failure at verifying passwords: {}", e);
                        return false;
                    }
                }
            } else {
                log::warn!("deny verifying password, because no password set for rowid={}", data.row.rowid);
                return false;
            }
        }

        // update usage
        let mut data = self.0.write().await;
        data.row.password_last_usage = Some(Utc::now());
        data.row.password_last_useragent = Some(user_agent);
        let pool = data.pool.clone();
        if let Err(e) = data.row.store(&pool).await {
            log::error!("failed to update password usage for rowid={}", data.row.rowid);
            return false;
        }

        return true;
    }
}


#[cfg(test)]
mod tests {
    use sqlx::SqlitePool;
    use super::*;
    use test_log::test;

    async fn get_pool() -> SqlitePool {
        let pool = sslo_lib::db::get_pool(None);
        sqlx::migrate!("../rsc/db_migrations/league_members").run(&pool).await.unwrap();
        return pool;
    }

    async fn create_new_item(pool: &SqlitePool) -> UserInterface {
        let row = UserDbRow::new(0);
        let data = UserData::new(pool, row);
        UserInterface::new(data)
    }

    async fn load_item_from_db(id: i64, pool: &SqlitePool) -> UserInterface {
        let mut row = UserDbRow::new(id);
        row.load(pool).await.unwrap();
        let data = UserData::new(&pool, row);
        UserInterface::new(data)
    }

    /// test item generation and property access
    #[test(tokio::test)]
    async fn new_item() {
        let pool = get_pool().await;

        // create item
        let row = UserDbRow::new(0);
        let data = UserData::new(&pool, row);
        let item = UserInterface::new(data);
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

        // set email
        let email_token = item.set_email("a.b@c.de".to_string()).await.unwrap();
        assert_eq!(item.email().await, None);
        assert!(item.verify_email(email_token).await);
        assert_eq!(item.email().await, Some("a.b@c.de".to_string()));

        // check if stored into db correctly
        let item = load_item_from_db(item.id().await, &pool).await;
        assert_eq!(item.email().await, Some("a.b@c.de".to_string()));
    }

    #[test(tokio::test)]
    async fn password() {

        // create item
        let pool = get_pool().await;
        let mut item = create_new_item(&pool.clone()).await;
        assert_eq!(item.email().await, None);

        // set password
        assert!(item.update_password(None, Some("unsecure_test_password".to_string())).await);
        assert!(item.verify_password("unsecure_test_password".to_string(), "unit test".to_string()).await);

        // check if stored into db correctly
        let mut item = load_item_from_db(item.id().await, &pool).await;
        assert!(item.verify_password("unsecure_test_password".to_string(), "unit test".to_string()).await);

        // update without old password must fail
        assert!(!item.update_password(None, Some("unsecure_updated_test_password".to_string())).await);

        // update password
        assert!(item.update_password(Some("unsecure_test_password".to_string()), Some("unsecure_updated_test_password".to_string())).await);

        // check if stored into db correctly
        let item = load_item_from_db(item.id().await, &pool).await;
        assert!(item.verify_password("unsecure_updated_test_password".to_string(), "unit test".to_string()).await);

        // verify wrong password must fail
        assert!(!item.verify_password("foobar".to_string(), "unit test".to_string()).await);
    }
}