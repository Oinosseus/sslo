use std::sync::{Arc, Weak};
use chrono::{DateTime, Utc};
use sslo_lib::db::PoolPassing;
use crate::user_grade;

pub struct Item {
    pool_ref_2parent: Weak<dyn PoolPassing>,
    row: super::row::Row,
}

impl Item {

    pub fn id(&self) -> i64 { self.row.rowid}

    /// Set up an object from a data row (assumed to be clean retrieved from db)
    pub(super) fn from_row(pool_ref: Weak<dyn PoolPassing>, row: super::row::Row) -> Arc<Self> {
        Arc::new(Self { pool_ref_2parent: pool_ref, row } )
    }

    /// datetime of last driven lap
    pub fn last_lap(&self) -> Option<DateTime<Utc>> {
        self.row.last_lap
    }

    /// name of the user
    pub fn name_ref(&self) -> &str {
        &self.row.name
    }

    pub fn promotion(&self) -> user_grade::Promotion {
        self.row.promotion.clone()
    }

    pub fn promotion_authority(&self) -> user_grade::PromotionAuthority {
        self.row.promotion_authority.clone()
    }

    /// Test if the user has a password set
    pub fn has_password(&self) -> bool {
        self.row.password.is_some()
    }

    /// database rowid
    pub fn rowid(&self) -> i64 {
        self.row.rowid
    }

}
