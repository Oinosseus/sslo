use axum::extract::{FromRef, FromRequestParts};
use axum::http::header;
use axum::http::request::Parts;
use crate::app_state::AppState;
use crate::user_grade::UserGrade;


/// Representing the current user of the http service
pub struct HttpUser {
    pub user_item: Option<crate::db::members::users::Item>,
    pub user_grade: UserGrade,
    pub cookie_login_item: Option<crate::db::members::cookie_logins::Item>,
}


impl HttpUser {

    pub fn name(&self) -> &str {
        if let Some(item) = &self.user_item {
            &item.name
        } else {
            ""
        }
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


        // try finding database user from cookies
        let mut user_item: Option<crate::db::members::users::Item> = None;
        let mut cookie_login_item: Option<crate::db::members::cookie_logins::Item> = None;
        for cookie_header in parts.headers.get_all(header::COOKIE) {
            if let Ok(cookie_string) = cookie_header.to_str() {
                if let Some(user_agent) = parts.headers.get(header::USER_AGENT) {
                    if let Ok(user_agent) = user_agent.to_str() {
                        if let Some(cli) = app_state.db_members.tbl_cookie_logins.from_cookie(user_agent, cookie_string).await {
                            user_item = app_state.db_members.tbl_users.from_id(cli.user).await;
                            cookie_login_item = Some(cli);
                            break;
                        }
                    }
                }
            }
        };

        let http_user = HttpUser {
            user_grade: UserGrade::from_user(&app_state, &user_item).await,
            user_item,
            cookie_login_item,
        };

        // return
        Ok(Self(http_user))
    }
}
