use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use crate::app_state::AppState;
use crate::http::http_user::HttpUserExtractor;
use super::GeneralError;

#[derive(Serialize)]
pub struct EmptyResponse {
}


#[derive(Deserialize)]
pub struct SetNameRequest {
    new_name: Option<String>,
    old_password: Option<String>,
    new_password: Option<String>,
}

pub async fn handler_update_settings(State(app_state): State<AppState>,
                                     HttpUserExtractor(http_user): HttpUserExtractor,
                                     Json(input): Json<SetNameRequest>) -> Response {

    if let Some(mut some_user) = http_user.user {

        // name
        if let Some(new_name) = input.new_name {
            match some_user.update_name(new_name).await {
                Ok(_) => {},
                Err(e) => {
                    log::error!("Could not update username: {}", e);
                    return GeneralError::new("Updating name failed".to_string(), "".to_string()).into_response();
                }
            };
        }

        // password
        if let Some(new_password) = input.new_password {
            match some_user.update_password(input.old_password, new_password).await {
                Ok(_) => {},
                Err(e) => {
                    log::error!("Failed to update password!");
                    return GeneralError::new("Updating password failed".to_string(), "".to_string()).into_response();
                }
            }
        }

    } else {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    Json(EmptyResponse{}).into_response()
}
