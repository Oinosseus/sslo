use std::ops::Sub;
use crate::app_state::AppState;
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
    pub fn label(&self) -> &'static str {
        match self {
            Self::None => "Pedestrian",
            Self::Obsolete => "Veteran",
            Self::Recent => "Driver",
        }
    }
}


#[derive(PartialEq, Clone)]
#[derive(sqlx::Type)]
#[derive(Debug)]
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


#[derive(PartialEq, Clone)]
#[derive(sqlx::Type)]
#[derive(Debug)]
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
    pub login_activity: LoginActivity,
    pub driving_activity: DrivingActivity,
    pub promotion: Promotion,
    pub promotion_authority: PromotionAuthority,
    pub is_root: bool,
}


impl UserGrade {

    /// create a new object with lowest permissions
    pub fn new_lowest() -> Self {
        Self {
            login_activity: LoginActivity::None,
            driving_activity: DrivingActivity::None,
            promotion: Promotion::None,
            promotion_authority: PromotionAuthority::Executing,
            is_root: false,
        }
    }

    pub async fn from_user(app_state: &AppState,
                           user: &Option<db::members::users::User>
    ) -> Self {

        // extract grade from database item
        if let Some(some_user) = user {

            // determine login activity
            let mut login_activity = LoginActivity::None;
            let last_cookie = app_state.db_members.cookie_login_from_last_usage(some_user).await;
            if let Some(some_last_cookie) = last_cookie {
                login_activity = match some_last_cookie.last_usage() {
                    None => { LoginActivity::None },
                    Some(last_login) => {
                        let obsolescence_threshold = chrono::Utc::now().sub(chrono::Duration::days(i64::from(app_state.config.general.days_recent_activity)));
                        if last_login > obsolescence_threshold { LoginActivity::Recent }
                        else { LoginActivity::Obsolete }
                    }
                };
            }

            // determine driving activity
            let driving_activity = match some_user.last_lap() {
                None => DrivingActivity::None,
                Some(last_lap) => {
                    let obsolescence_threshold = chrono::Utc::now().sub(chrono::Duration::days(i64::from(app_state.config.general.days_recent_activity)));
                    if last_lap > obsolescence_threshold { DrivingActivity::Recent }
                    else { DrivingActivity::Obsolete }
                }
            };

            // check for root
            let is_root: bool = match app_state.config.general.root_user_id {
                None => false,
                Some(root_user_id) => some_user.rowid() == root_user_id
            };

            Self {
                login_activity,
                driving_activity,
                promotion: some_user.promotion(),
                promotion_authority: some_user.promotion_authority(),
                is_root,
            }

        // assume lowest grade if no database item is available
        } else {
            Self::new_lowest()
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
