use std::error::Error;
use std::ops::Sub;
use sqlx::{Database, Decode, Encode, Sqlite};
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use chrono::{DateTime, Utc};
use crate::config::Config;
use crate::db;


#[derive(PartialEq)]
pub enum LoginActivity {
    None = 0,
    Obsolete = 1,
    Recent = 2,
}

impl LoginActivity {
    pub fn label(&self) -> &'static str {
        match self {
            Self::None => "Wildcard",
            Self::Obsolete => "Ghost",
            Self::Recent => "League",
        }
    }
}


#[derive(PartialEq)]
pub enum DrivingActivity {
    None = 0,
    Obsolete = 1,
    Recent = 2,
}

impl DrivingActivity {
    pub fn from_user(config: &Config, user_item: &db::members::users::Item) -> Self {
        let obsolescence_threshold = chrono::Utc::now().sub(chrono::Duration::days(i64::from(config.general.days_until_recent_activity_driving)));
        match user_item.last_lap {
            None => Self::None,
            Some(last_lap) => {
                if last_lap > obsolescence_threshold { Self::Obsolete }
                else { Self::Recent }
            }
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::None => "Pedestrian",
            Self::Obsolete => "Veteran",
            Self::Recent => "Driver",
        }
    }
}


#[derive(PartialEq)]
#[derive(sqlx::Type)]
#[repr(u32)]
pub enum PromotionAuthority {

    /// Only executing his promotion
    Executing = 0,

    /// Can also promote other users
    Chief = 1,
}

impl PromotionAuthority {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Executing => "Executing",
            Self::Chief => "Chief",
        }
    }
}


#[derive(PartialEq)]
#[derive(sqlx::Type)]
#[repr(u32)]
pub enum Promotion {
    None = 0,         // no further user rights
    Steward = 1,      // graceful server control
    Marshal = 2,      // force server control, update downloads
    Officer = 3,      // schedule races
    Commissar = 4,    // correct results, pronounce penalties
    Director = 5,     // manage series, edit presets
    Admin = 6,        // almost all permissions (except root)
}

impl Promotion {

    pub fn label(&self) -> &'static str {
        match self {
            Self::None => "",
            Self::Steward => "Steward",
            Self::Marshal => "Marshal",
            Self::Officer => "Officer",
            Self::Commissar => "Commissar",
            Self::Director => "Director",
            Self::Admin => "Administrator",
        }
    }
}


pub struct UserGrade {
    login_activity: LoginActivity,
    driving_activity: DrivingActivity,
    promotion: Promotion,
    promotion_authority: PromotionAuthority,
    is_root: bool,
}


impl UserGrade {

    pub fn from_user(config: &Config,
                     user_item: Option<db::members::users::Item>
    ) -> Self {

        // extract grade from database item
        if let Some(user_item) = user_item {

            // determine login activity
            let login_activity = match user_item.last_lap {
                None => LoginActivity::None,
                Some(last_lap) => {
                    let obsolescence_threshold = chrono::Utc::now().sub(chrono::Duration::days(i64::from(config.general.days_until_recent_activity_login)));
                    if last_lap > obsolescence_threshold { LoginActivity::Obsolete }
                    else { LoginActivity::Recent }
                }
            };

            // determine driving activity
            let driving_activity = match user_item.last_lap {
                None => DrivingActivity::None,
                Some(last_lap) => {
                    let obsolescence_threshold = chrono::Utc::now().sub(chrono::Duration::days(i64::from(config.general.days_until_recent_activity_driving)));
                    if last_lap > obsolescence_threshold { DrivingActivity::Obsolete }
                    else { DrivingActivity::Recent }
                }
            };

            // check for root
            let is_root: bool = match config.general.root_user_id {
                None => false,
                Some(root_user_id) => user_item.rowid == root_user_id
            };

            Self {
                login_activity,
                driving_activity,
                promotion: user_item.promotion,
                promotion_authority: user_item.promotion_authority,
                is_root,
            }

        // assume lowest grade if no database item is available
        } else {
            Self {
                login_activity: LoginActivity::None,
                driving_activity: DrivingActivity::None,
                promotion: Promotion::None,
                promotion_authority: PromotionAuthority::Executing,
                is_root: false,
            }
        }
    }


    pub fn label(&self) -> String {
        if self.is_root {
            "Root".to_string()
        } else if self.promotion == Promotion::None {
            format!("{} {}",
                    self.login_activity.label(),
                    self.driving_activity.label(),
            )
        } else {
            format!("{} {}, {} {}",
                    self.login_activity.label(),
                    self.driving_activity.label(),
                    self.promotion_authority.label(),
                    self.promotion.label(),
            )
        }
    }
}
