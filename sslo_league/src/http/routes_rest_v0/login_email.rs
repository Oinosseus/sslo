use axum::extract::State;
use axum::http::header::{REFRESH, SET_COOKIE};
use axum::response::Response;
use axum::Json;
use axum::response::IntoResponse;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use crate::app_state::AppState;
use crate::http::http_user::HttpUserExtractor;

#[derive(Deserialize)]
pub struct RequestData {
    email: String,
    password: Option<String>,
}

#[derive(Serialize)]
pub struct ResponseData {
}

pub async fn handler(State(app_state): State<AppState>,
                     HttpUserExtractor(http_user): HttpUserExtractor,
                     Json(input): Json<RequestData>,
) -> Response {

    // artificial slowdown
    let wait_ms: u64 = 1000u64 + u64::from(rand::thread_rng().next_u32()) / 0x200_000u64; // should result in ~2000 maximum
    tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;

    // create new token
    let mut cookie : Option<String> = None;
    if let Some(password) = input.password {
        if let Some(some_user) = app_state.db_members.user_from_email_password(http_user.user_agent.clone(), &input.email, password).await {
            cookie = app_state.db_members.cookie_login_new(&some_user).await;
        }
    }

    // done
    let mut response = Json(ResponseData{}).into_response();
    if let Some(cookie) = cookie {
        response.headers_mut().insert(SET_COOKIE, cookie.parse().unwrap());
    }
    response
}
