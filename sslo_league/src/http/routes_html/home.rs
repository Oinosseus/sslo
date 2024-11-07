use axum::http::StatusCode;
use axum::response::IntoResponse;
use crate::http::HtmlTemplate;
use crate::http::http_user::HttpUserExtractor;

pub async fn handler(HttpUserExtractor(http_user): HttpUserExtractor,) -> Result<impl IntoResponse, StatusCode> {
    let mut template = HtmlTemplate::new(http_user);
    template.push_body("Hello World!");
    Ok(template)
}
