use axum::extract::State;
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

pub async fn handler(State(app_state): State<AppState>, Json(input): Json<RequestData>) -> Json<ResponseData> {

    // artificial slowdown for 2..5s
    let wait_ms: u64 = 1000u64 + u64::from(rand::thread_rng().next_u32()) / 0x200_000u64; // should result in ~2000 maximum
    tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;

    // create new token
    let mut token : Option<String> = None;  // need this option, because build fails when nesting new_email_login_token() and send_email()
    if let Some(mut user_item) = app_state.db_members.user_from_email(&input.email).await {
        token = user_item.update_email_login_token().await;
    }

    // send info email
    if let Some(t) = token {
        let message = format!("Use this token: '{}'", t);
        if let Err(e) = crate::helpers::send_email(&app_state.config, &input.email, "Email Login", &message).await {
            log::warn!("Could not create new email token for '{}': {}", &input.email, e)
        }
    }

    ResponseData{}.into()
}
