use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
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
    println!("wait_ms: {}", wait_ms);
    tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;

    // create new token
    let token = app_state.db_members.new_email_login_token(&input.email)
        .await
        .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;

    // send information email
    let mut message = format!("Use this token: '{}'", token);
    let _ = crate::helpers::send_email(&app_state.config,  &input.email, "Email Login", message);

    Ok(ResponseData{}.into())
}
