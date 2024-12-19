use chrono::{DateTime, Utc};
use crate::user_grade;

#[derive(sqlx::FromRow)]
pub(super) struct Row {
    pub rowid: i64,
    pub name: String,
    pub promotion_authority: user_grade::PromotionAuthority,
    pub promotion: user_grade::Promotion,
    pub last_lap: Option<DateTime<Utc>>,
    pub email: Option<String>,
    pub email_token: Option<String>,
    pub email_token_creation: Option<DateTime<Utc>>,
    pub email_token_consumption: Option<DateTime<Utc>>,
    pub password: Option<String>,
    pub password_last_usage: Option<DateTime<Utc>>,
    pub password_last_useragent: Option<String>,
}
