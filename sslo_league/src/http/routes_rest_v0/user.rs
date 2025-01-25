use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use crate::app_state::AppState;
use crate::http::http_user::HttpUserExtractor;
use super::GeneralError;

#[derive(Serialize)]
pub struct EmptyResponse {
}


#[derive(Deserialize)]
pub struct SetNameRequest {
    name: String,
}

pub async fn handler_set_name(State(_app_state): State<AppState>,
                              HttpUserExtractor(mut http_user): HttpUserExtractor,
                              Json(input): Json<SetNameRequest>) -> Response {

    if !http_user.is_logged_in() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    // tokio::time::sleep(std::time::Duration::from_millis(5000)).await;

    match http_user.user.set_name(input.name).await {
        Ok(_) => {},
        Err(e) => {
            log::error!("Could not update username: {}", e);
            return GeneralError::new("Updating name failed".to_string(), "".to_string()).into_response();
        }
    };

    Json(EmptyResponse{}).into_response()
}


#[derive(Deserialize)]
pub struct UpdateSettingsRequest {
    new_name: Option<String>,
    old_password: Option<String>,
    new_password: Option<String>,
}

pub async fn handler_update_settings(State(_app_state): State<AppState>,
                                     HttpUserExtractor(mut http_user): HttpUserExtractor,
                                     Json(input): Json<UpdateSettingsRequest>) -> Response {

    if !http_user.is_logged_in() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    // name
    if let Some(new_name) = input.new_name {
        match http_user.user.set_name(new_name).await {
            Ok(_) => {},
            Err(e) => {
                log::error!("Could not update username: {}", e);
                return GeneralError::new("Updating name failed".to_string(), "".to_string()).into_response();
            }
        };
    }

    // password
    if let Some(new_password) = input.new_password {
        if !http_user.user.update_password(input.old_password, Some(new_password)).await {
            log::error!("Failed to update password!");
            return GeneralError::new("Updating password failed".to_string(), "".to_string()).into_response();
        }
    }

    Json(EmptyResponse{}).into_response()
}
