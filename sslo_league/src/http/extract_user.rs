use axum::extract::{FromRef, FromRequestParts};
use axum::http::{header, StatusCode};
use axum::http::request::Parts;
use axum::response::IntoResponse;
use crate::app_state::AppState;

pub struct WebsiteUser (pub String);

#[axum::async_trait]
impl<S> FromRequestParts<S> for WebsiteUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {

        let app_state = AppState::from_ref(state);

        // println!("HERE1");

        // for hmap in parts.headers.iter() {
        //     println!("Header[{}] = {}", hmap.0, hmap.1.to_str().unwrap());
        // }

        // try all cookies
        for cookie_header in parts.headers.get_all(header::COOKIE) {
            if let Ok(cookie_string) = cookie_header.to_str() {
                if let Some(foo) = app_state.db_members.tbl_cookie_logins.from_cookie(cookie_string).await {
                    println!("Cookie found: {}", foo.rowid);
                    break;
                }
            }
        };

        Ok(Self("HelloExtractor".to_string()))
    }
}