use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use crate::app_state::AppState;
use crate::http::http_user::HttpUserExtractor;


#[derive(Deserialize)]
pub struct ChangeNameRequest {
    new_name: String,
}

#[derive(Serialize)]
pub struct EmptyResponse {
}

pub async fn handler_set_name(State(app_state): State<AppState>,
                              HttpUserExtractor(http_user): HttpUserExtractor,
                              Json(input): Json<ChangeNameRequest>) -> Response {

    if let Some(mut user_item) = http_user.user_item {
        match user_item.update_name(input.new_name).await {
            Ok(_) => {},
            Err(e) => {
                log::error!("Could not update username: {}", e);
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };
    } else {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    Json(EmptyResponse{}).into_response()
}
