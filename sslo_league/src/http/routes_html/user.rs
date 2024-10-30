use axum::extract::{OriginalUri, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use crate::app_state::AppState;
use crate::http::HtmlTemplate;
use crate::http::http_user::HttpUserExtractor;


pub async fn handler_settings(State(app_state): State<AppState>,
                              OriginalUri(uri): OriginalUri,
                              HttpUserExtractor(http_user): HttpUserExtractor) -> Result<impl IntoResponse, StatusCode> {

    // require login
    if http_user.user_item.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let mut html = HtmlTemplate::new(http_user);
    html.include_js("/rsc/js/user.js");

    // change form
    html.push_body("<form acton=\"\" method=\"POST\" class=\"GridForm\" id=\"UserSettingsForm\">");
    html.push_body(&format!("<label>Name:</label><input type=\"text\" placeholder=\"name\" name=\"new_name\" value=\"{}\">",
                            html.http_user().name()));
    html.push_body("<label></label><button type=\"submit\">Save</button>");
    html.push_body("</form>");

    Ok(html)
}
