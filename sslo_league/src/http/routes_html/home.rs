use axum::http::StatusCode;
use axum::response::Response;
use crate::http::HtmlTemplate;
use crate::http::http_user::HttpUserExtractor;

pub async fn handler(HttpUserExtractor(http_user): HttpUserExtractor,) -> Result<Response, StatusCode> {
    let mut template = HtmlTemplate::new(http_user);
    template.push_body("Hello World!");
    Ok(template.into_response().await)
}
