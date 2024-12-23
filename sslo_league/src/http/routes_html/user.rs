use axum::extract::{OriginalUri, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use crate::app_state::AppState;
use crate::http::HtmlTemplate;
use crate::http::http_user::HttpUserExtractor;


pub async fn handler_settings(State(_app_state): State<AppState>,
                              OriginalUri(_uri): OriginalUri,
                              HttpUserExtractor(http_user): HttpUserExtractor) -> Result<impl IntoResponse, StatusCode> {

    if http_user.user.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let mut html = HtmlTemplate::new(http_user);
    html.include_js("/rsc/js/user.js");

    // change form
    html.push_body("<form acton=\"\" method=\"POST\" class=\"GridForm\" id=\"UserSettingsForm\">");

    // name
    html.push_body(&format!("<label>Name:</label><input type=\"text\" placeholder=\"name\" name=\"new_name\" value=\"{}\">", html.http_user().name()));

    // password
    html.push_body("<label>Current Password:</label><input type=\"password\" placeholder=\"current password\" name=\"old_password\">");
    html.push_body("<label>New Password:</label><input type=\"password\" placeholder=\"new password\" name=\"new_password1\">");
    html.push_body("<label>Verify Password:</label><input type=\"password\" placeholder=\"verify password\" name=\"new_password2\">");

    // save
    html.push_body("<label></label><button type=\"submit\">Save</button>");
    html.push_body("</form>");

    Ok(html)
}
