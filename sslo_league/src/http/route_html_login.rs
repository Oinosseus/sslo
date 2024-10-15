use axum::http::StatusCode;
use axum::response::IntoResponse;
use crate::http::HtmlTemplate;

pub async fn handler() -> Result<impl IntoResponse, StatusCode> {
    let mut html = HtmlTemplate::new();
    html.include_css("/rsc/css/login.css");
    html.include_js("/rsc/js/login.js");

    // Tab Selection
    html.push_body("<div id=\"TabSelection\">");
    html.push_body("<div>Choose Login Method:</div>");
    html.push_body("<button id=\"LoginButtonLoginPassword\" onclick=\"tabSelectByIndex(0)\" class=\"ActiveButton\">Password</button>");
    html.push_body("<button id=\"LoginButtonLoginEmail\" onclick=\"tabSelectByIndex(1)\">Email</button>");
    html.push_body("<button id=\"LoginButtonLoginSteam\" onclick=\"tabSelectByIndex(2)\">Steam</button>");
    html.push_body("<button id=\"RegisterButton\" onclick=\"tabSelectByIndex(3)\">Register</button>");
    html.push_body("</div>");

    // Login with Password
    html.push_body("<form id=\"TabLoginPassword\" class=\"ActiveTab\" method=\"post\">");
    html.push_body("<label>Login with SSLO Password</label>");
    html.push_body("<input required autofocus placeholder=\"email\" type=\"email\" name=\"LoginEmail\">");
    html.push_body("<input required placeholder=\"password\" type=\"password\" name=\"LoginPassword\">");
    html.push_body("<button type=\"submit\">Login</button>");
    html.push_body("</form>");

    // Login with Email SSO
    html.push_body("<form id=\"TabLoginEmail\" method=\"post\">");
    html.push_body("<label>Login via sending Email login link</label>");
    html.push_body("<input required autofocus placeholder=\"email\" type=\"email\" name=\"LoginEmail\">");
    html.push_body("<button type=\"submit\">Send Login Link</button>");
    html.push_body("</form>");

    // Login with Steam SSO
    html.push_body("<form id=\"TabLoginSteam\" method=\"post\">");
    html.push_body("<label>Login via Steam</label>");
    html.push_body("<button type=\"submit\">Forward to Steam Login</button>");
    html.push_body("</form>");

    // SSLO Local Registration
    html.push_body("<form id=\"TabRegisterSsloForm\" method=\"post\">");
    html.push_body("<label>Register with email address</label>");
    html.push_body("<input required autofocus placeholder=\"email\" type=\"email\" name=\"LoginEmail\">");
    html.push_body("<button type=\"submit\">Register</button>");
    html.push_body("</form>");

    return Ok(html);
}
