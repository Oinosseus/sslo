use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct JsonResponse {
    foo: String,
}

pub async fn handler() -> Result<Json<JsonResponse>, StatusCode> {
    Ok(JsonResponse {
        foo: String::from("foo"),
    }.into())
}
