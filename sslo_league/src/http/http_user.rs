use std::ops::Sub;
use axum::extract::{FromRef, FromRequestParts};
use axum::http::header;
use axum::http::request::Parts;
use crate::app_state::AppState;
use crate::user_grade::UserGrade;


/// Representing the current user of the http service
pub struct HttpUser {
    pub name: String,
    pub user_grade: UserGrade,
}


impl HttpUser {

    /// create a new unknown/guest visitor (aka. pedestrian)
    pub fn new_pedestrian() -> Self {
        Self {
            name: "Pedestrian".to_string(),
            user_grade: UserGrade::Pedestrian,
        }
    }


    pub fn from_user_item(app_state: &AppState,
                          user_item: &crate::db::members::users::Item
    ) -> Self {

        // return new item
        Self {
            name: user_item.name.to_string(),
            user_grade: UserGrade::from_user(&app_state.config, user_item),
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
        let user_item = match user_item {  // extract user_item or return pedestrian
            Some(x) => x,
            None => return Ok(Self(HttpUser::new_pedestrian())),
        };

        // return
        let http_user = HttpUser::from_user_item(&app_state, &user_item);
        Ok(Self(http_user))
    }
}
