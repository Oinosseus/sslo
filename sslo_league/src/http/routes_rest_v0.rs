use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;

pub mod login_email;
pub mod user;



#[derive(serde::Serialize)]
pub struct GeneralError {
    summary: String,
    description: String,
}

impl GeneralError {
    pub fn new(summary: String, description: String) -> Self {
        Self{summary, description}
    }
}

impl IntoResponse for GeneralError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(self)).into_response()
    }
}
