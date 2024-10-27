use axum::extract::{FromRef, FromRequestParts};
use axum::http::{header, StatusCode};
use axum::http::request::Parts;
use crate::app_state::AppState;

/// Representing the current user of the http service
pub struct HttpUser {
    pub name: String,
}

impl HttpUser {

    /// create a new unknown/guest visitor (aka. pedestrian)
    pub fn new_pedestrian() -> Self {
        Self {
            name: "Pedestrian".to_string()
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
        for cookie_header in parts.headers.get_all(header::COOKIE) {
            if let Ok(cookie_string) = cookie_header.to_str() {
                if let Some(cookie_login_item) = app_state.db_members.tbl_cookie_logins.from_cookie(cookie_string).await {
                    user_item = app_state.db_members.tbl_users.from_id(cookie_login_item.user).await;
                    break;
                }
            }
        };

        // return
        let http_user = match user_item {
            Some(item) => HttpUser{name: item.name.to_string()},
            None => HttpUser::new_pedestrian(),
        };
        Ok(Self(http_user))
    }
}