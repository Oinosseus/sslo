use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use crate::http::http_user::HttpUser;

pub mod login_email;
pub mod user;



pub struct GeneralError {
    status_code: StatusCode,
    description: String,
}

#[derive(serde::Serialize)]
pub struct GeneralErrorJson {
    summary: String,
    description: String,
}

impl GeneralError {
    pub fn new(status_code: StatusCode, description: String) -> Self {
        Self {status_code, description}
    }
}

impl IntoResponse for GeneralError {
    fn into_response(self) -> axum::response::Response {
        let json_data = GeneralErrorJson {
            summary: self.status_code.canonical_reason().unwrap_or("unknown").to_string(),
            description: self.description
        };
        (self.status_code, Json(json_data)).into_response()
    }
}
