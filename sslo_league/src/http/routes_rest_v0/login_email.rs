use axum::extract::State;
use axum::http::header::{SET_COOKIE};
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
    let wait_ms: u64 = 1000u64 + u64::from(rand::thread_rng().next_u32()) / 0x200_000u64; // should result in ~1..3s
    tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;

    // get db_obsolete tables
    let tbl_usr = app_state.database.db_members().await.tbl_users().await;
    let tbl_cookie = app_state.database.db_members().await.tbl_cookie_logins().await;

    // create new token
    let mut cookie : Option<String> = None;
    if let Some(password) = input.password {
        if let Some(some_user) = tbl_usr.user_by_email(&input.email).await {
            if some_user.verify_password(password, http_user.user_agent.clone()).await {
                if let Some(login_cookie_item) = tbl_cookie.create_new_cookie(&some_user).await {
                    cookie = login_cookie_item.get_cookie().await;
                }
            }
        }
    }

    // done
    let mut response = Json(ResponseData{}).into_response();
    if let Some(cookie) = cookie {
        response.headers_mut().insert(SET_COOKIE, cookie.parse().unwrap());
    }
    response
}
