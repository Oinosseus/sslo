use axum::http::StatusCode;
use axum::response::IntoResponse;
use crate::http::HtmlTemplate;

pub async fn handler() -> Result<impl IntoResponse, StatusCode> {
    let mut html = HtmlTemplate::new();
    html.include_css("/rsc/css/login.css");

    html.push_body("Local login");
    html.push_body("<form method=\"post\">");
    html.push_body("<input required autofocus placeholder=\"username\" type=\"text\" name=\"LoginUsername\">");
    html.push_body("<input required placeholder=\"password\" type=\"password\" name=\"LoginUsername\">");
    html.push_body("<button type=\"submit\">Login</button>");
    html.push_body("</form>");

    return Ok(html);
}
