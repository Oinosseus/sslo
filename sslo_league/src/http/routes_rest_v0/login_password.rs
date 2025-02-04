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
    email: Option<String>,
    user_id: Option<i64>,
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

    // general login failed response
    let response_failed = GeneralError::new(StatusCode::INTERNAL_SERVER_ERROR, "Login Failed!".to_string()).into_response();

    // get db_obsolete tables
    let tbl_usr = app_state.database.db_members().await.tbl_users().await;
    let tbl_eml = app_state.database.db_members().await.tbl_email_accounts().await;
    let tbl_cookie = app_state.database.db_members().await.tbl_cookie_logins().await;

    // identify user
    let user: UserItem = match input.user_id {

        // by user-id
        Some(user_id) => match tbl_usr.user_by_id(user_id).await {
            None => {
                log::warn!("Login via password failed because no user for rowid={} found!", user_id);
                return response_failed;
            },
            Some(user) => user,
        },

        // by email
        None => match input.email {
            None => {
                log::warn!("Login via password failed because neither user-id nor email provided!");
                return response_failed;
            },
            Some(email_addr) => match tbl_eml.item_by_email(&email_addr).await {
                None => {
                    log::warn!("Login via password failed because unknown email '{}'!", email_addr);
                    return response_failed;
                },
                Some(eml) => match eml.user().await {
                    Some(usr) => usr,
                    None => {
                        log::warn!("Login via password failed because email '{}' is not assigned to a user!", email_addr);
                        return response_failed;
                    }
                }
            }
        },
    };

    // verify password
    match input.password {
        None => {
            log::warn!("Deny login of {} because no password given!", user.display().await);
            return response_failed;
        },
        Some(password) => {
            if !user.verify_password(password, http_user.user_agent.clone()).await {
                log::warn!("Deny login of {} because password cannot be verified!", user.display().await);
                return response_failed;
            }
        }
    }

    // create new token
    let mut cookie : Option<String> = None;
    if let Some(login_cookie_item) = tbl_cookie.create_new_cookie(&user).await {
        cookie = login_cookie_item.get_cookie().await;
    }

    // done
    let mut response = Json(ResponseData{}).into_response();
    if let Some(cookie) = cookie {
        response.headers_mut().insert(SET_COOKIE, cookie.parse().unwrap());
    }
    response
}
