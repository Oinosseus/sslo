use axum::extract::{FromRef, FromRequestParts};
use axum::http::header;
use axum::http::request::Parts;
use crate::app_state::AppState;
use crate::user_grade::UserGrade;


/// Representing the current user of the http service
pub struct HttpUser {
    pub user: Option<crate::db::members::users::User>,
    pub user_grade: UserGrade,
    pub cookie_login: Option<crate::db::members::cookie_logins::CookieLogin>,
    pub user_agent: String,
}


impl HttpUser {

    /// Crate a object with lowest permissions
    pub fn new_lowest() -> Self {
        Self {
            user: None,
            user_grade: UserGrade::new_lowest(),
            cookie_login: None,
            user_agent: "".to_string(),
        }
    }

    pub fn name(&self) -> &str {
        if let Some(item) = &self.user {
            &item.name_ref()
        } else {
            ""
        }
    }


    pub fn user(&self) -> Option<&crate::db::members::users::User> { self.user.as_ref()}

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

        // try finding database user from cookies
        let mut user: Option<crate::db::members::users::User> = None;
        let mut cookie_login: Option<crate::db::members::cookie_logins::CookieLogin> = None;
        for cookie_header in parts.headers.get_all(header::COOKIE) {
            if let Ok(cookie_string) = cookie_header.to_str() {
                if let Some(cl) = app_state.db_members.cookie_login_from_cookie(user_agent.to_string(), cookie_string).await {
                    if let Some(cl_user) = cl.user().await {
                        user = Some(cl_user);
                        cookie_login = Some(cl);
                        break;
                    }
                }
            }
        };

        let http_user = HttpUser {
            user_grade: UserGrade::from_user(&app_state, &user).await,
            user,
            cookie_login,
            user_agent,
        };

        // return
        Ok(Self(http_user))
    }
}
