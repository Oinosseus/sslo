use axum::extract::State;
use axum::http::header::{SET_COOKIE};
use axum::http::StatusCode;
use axum::response::Response;
use axum::Json;
use axum::response::IntoResponse;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use crate::app_state::AppState;
use crate::db2::members::users::UserItem;
use crate::http::http_user::HttpUserExtractor;
use crate::http::routes_rest_v0::GeneralError;

#[derive(Deserialize)]
pub struct RequestData {
    identification: String,
    password: String,
}

#[derive(Serialize)]
pub struct ResponseData {
}

pub async fn handler(State(app_state): State<AppState>,
                     HttpUserExtractor(http_user): HttpUserExtractor,
                     Json(input): Json<RequestData>,
) -> Response {

    // artificial slowdown
    let wait_ms: u64 = 1000u64 + u64::from(rand::thread_rng().next_u32()) / 0x200_000u64; // results in 1000..3048ms
    tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;

    // general login failed response
    let response_failed = GeneralError::new(StatusCode::INTERNAL_SERVER_ERROR, "Login Failed!".to_string()).into_response();

    // get db tables
    let tbl_usr = app_state.database.db_members().await.tbl_users().await;
    let tbl_eml = app_state.database.db_members().await.tbl_email_accounts().await;
    let tbl_cookie = app_state.database.db_members().await.tbl_cookie_logins().await;

    // try to identify user by email
    let mut user: Option<UserItem> = None;
    if let Some(eml) = tbl_eml.item_by_email(&input.identification).await {
        user = eml.user().await;
        if user.is_none() {
            log::error!("Failed to retrieve user from {}", eml.display().await);
        }
    }

    // try to identify user from ID
    if let Ok(user_id) = input.identification.parse::<i64>() {
        user = tbl_usr.user_by_id(user_id).await;
    }

    // verify password
    let mut cookie : Option<String> = None;
    if let Some(user) = user {
        if !user.verify_password(input.password, http_user.user_agent.clone()).await {
            log::warn!("Deny login of {} because password cannot be verified!", user.display().await);
            return response_failed;
        }

        // create new token
        if let Some(login_cookie_item) = tbl_cookie.create_new_cookie(&user).await {
            cookie = login_cookie_item.get_cookie().await;
        }
    }

    // done
    let mut response = Json(ResponseData{}).into_response();
    if let Some(cookie) = cookie {
        response.headers_mut().insert(SET_COOKIE, cookie.parse().unwrap());
    }
    response
}
