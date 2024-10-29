use std::error::Error;
use std::ops::Sub;
use sqlx::{Database, Decode, Encode, Sqlite};
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use crate::config::Config;
use crate::db;

pub enum UserGrade {
    Pedestrian,
    Ghost,
    Guest,
    WildcardDriver,
    LeagueGhost,
    LeagueDriver,
    RacingSteward,
    LeagueMarshal,
    LeagueCommissar,
    LeagueDirector,
    ServerDirector,
    ServerAdmin,
}


impl UserGrade {

    pub fn from_user(config: &Config,
                     user: &db::members::users::Item) -> Self {

        #[derive(PartialEq)]
        enum Activity {
            None,
            Any,
            Recent,
        }

        // check for driving activity
        let mut lap_activity = Activity::None;
        let time_recent_login = chrono::Utc::now().sub(chrono::Duration::days(i64::from(config.general.days_until_recent_activity_login)));
        if let Some(last_lap) = user.last_lap {
            if last_lap < time_recent_login {
                lap_activity = Activity::Any;
            } else {
                lap_activity = Activity::Recent;
            }
        }

        // check for login activity
        let mut login_activity = Activity::None;
        let time_recent_driving = chrono::Utc::now().sub(chrono::Duration::days(i64::from(config.general.days_until_recent_activity_driving)));
        if let Some(last_login) = user.last_login {
            if last_login < time_recent_driving {
                login_activity = Activity::Any;
            } else {
                login_activity = Activity::Recent;
            }
        }

        // start grade from lowest
        let mut grade = Self::Pedestrian;

        if lap_activity != Activity::None {

        }

        // Server Admin
        if config.general.user_id_server_admin == user.rowid {
            return Self::ServerAdmin;

        // recent login and recent laps
        // -> most activity
        // -> only check promotions
        } else if lap_activity == Activity::Recent && login_activity == Activity::Recent {
            return match user.promotion {
                Promotion::ServerDirector => Self::ServerDirector,
                Promotion::LeagueDirector => Self::LeagueDirector,
                Promotion::LeagueCommissar => Self::LeagueCommissar,
                Promotion::LeagueMarshal => Self::LeagueMarshal,
                Promotion::RacingSteward => Self::RacingSteward,
                Promotion::None => Self::LeagueDriver,
            };

        } else if lap_activity == Activity::Recent && login_activity == Activity::Any {
        } else if lap_activity == Activity::Recent && login_activity == Activity::None {

        } else if lap_activity == Activity::Any && login_activity == Activity::Recent {
        } else if lap_activity == Activity::Any && login_activity == Activity::Any {
        } else if lap_activity == Activity::Any && login_activity == Activity::None {

        } else if lap_activity == Activity::None && login_activity == Activity::Recent {
        } else if lap_activity == Activity::None && login_activity == Activity::Any {
            return Self::Guest;

        } else if lap_activity == Activity::None && login_activity == Activity::None {
            return Self::Pedestrian;

        } else {
            log::error!("Got unexpected activity condition: {:?}, {:?}", lap_activity, login_activity);
            return Self::Pedestrian;
        }
    }

    pub fn as_str(&self) -> &'static str{
        match self {
            Self::Pedestrian => "Pedestrian",
            Self::Guest => "Guest",
            Self::WildcardDriver => "Wildcard Driver",
            Self::LeagueGhost => "League Ghost",
            Self::LeagueDriver => "League Driver",
            Self::RacingSteward => "Racing Steward",
            Self::LeagueMarshal => "League Marshal",
            Self::LeagueCommissar => "League Commissar",
            Self::LeagueDirector => "League Director",
            Self::ServerDirector => "Server Director",
            Self::ServerAdmin => "Server Admin",
        }
    }
}

#[derive(PartialEq)]
#[derive(sqlx::Type)]
pub enum Promotion {
    None = 0,
    RacingSteward = 1,
    LeagueMarshal = 2,
    LeagueCommissar = 3,
    LeagueDirector = 4,
    ServerDirector = 5,
}
