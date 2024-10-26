use axum::http::StatusCode;
use axum::response::IntoResponse;
use crate::http::HtmlTemplate;

pub async fn handler() -> Result<impl IntoResponse, StatusCode> {
    let mut template = HtmlTemplate::new();
    template.push_body("Hello World!");
    Ok(template)
}
