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

    match http_user.user.set_name(input.name).await {
        Ok(_) => {},
        Err(e) => {
            log::error!("Could not update username: {}", e);
            return GeneralError::new(StatusCode::INTERNAL_SERVER_ERROR,
                                     "Updating name failed".to_string()).into_response();
        }
    };

    Json(EmptyResponse{}).into_response()
}


#[derive(Deserialize)]
pub struct SetPasswordRequest {
    old_password: Option<String>,
    new_password: Option<String>,
}

pub async fn handler_set_password(State(_app_state): State<AppState>,
                                     HttpUserExtractor(mut http_user): HttpUserExtractor,
                                     Json(input): Json<SetPasswordRequest>) -> Response {

    if !http_user.is_logged_in() {
        return GeneralError::new(StatusCode::UNAUTHORIZED,
                                 "No user logged in".to_string(),
        ).into_response();
    }

    if let Some(new_password) = input.new_password {
        if !http_user.user.update_password(input.old_password, Some(new_password)).await {
            log::error!("Failed to update password!");
            return GeneralError::new(StatusCode::INTERNAL_SERVER_ERROR,
                                     "Updating password failed".to_string(),
            ).into_response();
        }
    }

    Json(EmptyResponse{}).into_response()
}
