use axum::http::StatusCode;
use axum::response::IntoResponse;
use crate::http::HtmlTemplate;

pub async fn handler() -> Result<impl IntoResponse, StatusCode> {
    let mut html = HtmlTemplate::new();
    html.include_css("/rsc/css/login.css");
    html.include_js("/rsc/js/login.js");

    // Tab Selection
    html.push_body("<div id=\"TabSelection\">");
    html.push_body("<button id=\"LoginSsloButton\">SSLO Login</button>");
    html.push_body("<button id=\"RegisterSsloButton\">Register</button>");
    html.push_body("</div>");


    // SSLO Local Login
    html.push_body("<div id=\"LoginSsloForm\">");
    html.push_body("<label>Local SSLO login</label>");
    html.push_body("<form method=\"post\">");
    html.push_body("<input required autofocus placeholder=\"username\" type=\"text\" name=\"LoginUsername\">");
    html.push_body("<input required placeholder=\"password\" type=\"password\" name=\"LoginPassword\">");
    html.push_body("<button type=\"submit\">Login</button>");
    html.push_body("</form>");
    html.push_body("</div>");

    // SSLO Local Registration
    html.push_body("<div id=\"RegisterSsloForm\"");
    html.push_body("<label>Local SSLO Registration</label>");
    html.push_body("<form method=\"post\">");
    html.push_body("<input required autofocus placeholder=\"username\" type=\"text\" name=\"LoginUsername\">");
    html.push_body("<input required placeholder=\"password\" type=\"password\" name=\"LoginPassword\">");
    html.push_body("<input required placeholder=\"verify password\" type=\"password\" name=\"LoginPassword2\">");
    html.push_body("<button type=\"submit\">Register</button>");
    html.push_body("</form>");
    html.push_body("</div>");


    return Ok(html);
}
