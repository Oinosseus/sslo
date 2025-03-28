use axum::extract::{FromRef, FromRequestParts};
use axum::http::header;
use axum::http::request::Parts;
use chrono::Utc;
use crate::app_state::AppState;
use crate::db2::members::users::Promotion;
use super::super::db2::members::users::UserItem;
use super::super::db2::members::cookie_logins::CookieLoginItem;

/// Representing the current user of the http service
pub struct HttpUser {
    pub user: UserItem,
    pub cookie_login: Option<CookieLoginItem>,
    pub user_agent: String,
}


impl HttpUser {

    /// Crate a object with lowest permissions
    pub async fn new_anonymous(app_state: AppState) -> Self {
        let tbl_usr = app_state.database.db_members().await.tbl_users().await;
        Self {
            user: tbl_usr.user_dummy().await,
            cookie_login: None,
            user_agent: "".to_string(),
        }
    }

    pub fn is_logged_in(&self) -> bool {
        self.cookie_login.is_some()
    }
}


/// Extractor for HttpUser to be used in route handlers
pub struct HttpUserExtractor(pub HttpUser);


#[axum::async_trait]
impl<S> FromRequestParts<S> for HttpUserExtractor
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ();

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {

        let app_state = AppState::from_ref(state);

        // extract user agent
        let mut user_agent = String::new();
        if let Some(some_user_agent) = parts.headers.get(header::USER_AGENT) {
            if let Ok(some_user_agent_string) = some_user_agent.to_str() {
                user_agent = some_user_agent_string.to_string();
            }
        }

        // get tables
        let tbl_cookie = app_state.database.db_members().await.tbl_cookie_logins().await;

        // try finding database user from cookies
        for cookie_header in parts.headers.get_all(header::COOKIE) {
            if let Ok(cookie_string) = cookie_header.to_str() {
                if let Some(cl) = tbl_cookie.item_by_cookie(user_agent.to_string(), cookie_string).await {
                    if let Some(mut cl_user) = cl.user().await {

                        // track user login
                        cl_user.set_last_login(Utc::now()).await;

                        // create http user
                        let http_user = HttpUser {
                            user: cl_user,
                            cookie_login: Some(cl),
                            user_agent,
                        };
                        return Ok(Self(http_user));
                    }
                }
            }
        };

        // create dummy user if no other user found
        let tbl_usr = app_state.database.db_members().await.tbl_users().await;
        let http_user = HttpUser {
            user: tbl_usr.user_dummy().await,
            cookie_login: None,
            user_agent,
        };
        Ok(Self(http_user))
    }
}
