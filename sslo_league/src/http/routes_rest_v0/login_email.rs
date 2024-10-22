use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use log::error;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use crate::app_state::AppState;

#[derive(Deserialize)]
pub struct RequestData {
    email: String,
}

#[derive(Serialize)]
pub struct ResponseData {
}

pub async fn handler(State(app_state): State<AppState>,
                     Json(input): Json<RequestData>,
) -> Result<Json<ResponseData>, StatusCode> {

    // artificial slowdown for 2..5s
    let wait_ms: u64 = 1000u64 + u64::from(rand::thread_rng().next_u32()) / 0x200_000u64; // should result in ~2000 maximum
    tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;

    // create new token
    let token = app_state.db_members.tbl_email.new_email_login_token(&input.email)
        .await
        .or_else(|e| {
            log::warn!("Could not create new email token for '{}': {}", &input.email, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        })?;

    // send information email
    let mut message = format!("Use this token: '{}'", token);
    if crate::helpers::send_email(&app_state.config, &input.email, "Email Login", message).await.is_err() {
        log::error!("Could not send registration email");
    }

    Ok(ResponseData{}.into())
}
