use axum::extract::{OriginalUri, State};
use axum::http::StatusCode;
use axum::response::Response;
use crate::app_state::AppState;
use crate::http::HtmlTemplate;
use crate::http::http_user::HttpUserExtractor;


pub async fn handler_settings(State(_app_state): State<AppState>,
                              HttpUserExtractor(http_user): HttpUserExtractor) -> Result<Response, StatusCode> {

    if !http_user.is_logged_in() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let mut html = HtmlTemplate::new(http_user);
    html.include_js("/rsc/js/user.js");

    // change form
    html.push_body("<form acton=\"\" method=\"POST\" class=\"GridForm\" id=\"UserSettingsForm\">");

    // name
    html.push_body(&format!("<label>Name:</label><input type=\"text\" placeholder=\"name\" name=\"new_name\" value=\"{}\">", html.http_user().user.name().await));

    // password
    html.push_body("<label>Current Password:</label><input type=\"password\" placeholder=\"current password\" name=\"old_password\">");
    html.push_body("<label>New Password:</label><input type=\"password\" placeholder=\"new password\" name=\"new_password1\">");
    html.push_body("<label>Verify Password:</label><input type=\"password\" placeholder=\"verify password\" name=\"new_password2\">");

    // save
    html.push_body("<label></label><button type=\"submit\">Save</button>");
    html.push_body("</form>");

    Ok(html.into_response().await)
}


pub async fn handler_profile(State(app_state): State<AppState>,
                             HttpUserExtractor(http_user): HttpUserExtractor) -> Result<Response, StatusCode> {

    let tbl_eml = app_state.database.db_members().await.tbl_email_accounts().await;

    let mut html = HtmlTemplate::new(http_user);
    html.include_css("/rsc/css/user.css");
    html.include_js("/rsc/js/user_profile.js");
    html.http_user();

    html.push_body("<div class=\"BgBox\"><table id=\"UserProfile\">");

    html.push_body("<tr><th>Id</th><td>");
    html.push_body(&format!("{}", html.http_user.user.id().await));
    html.push_body("</td></tr>");

    html.push_body("<tr><th>Name</th><td><div class=\"LiveInput\" id=\"ProfileUserName\"><input type=\"text\" value=\"");
    html.push_body(&html.http_user.user.html_name().await);
    html.push_body("\"><button title=\"Save\">&#128190;</button></div></td></tr>");

    html.push_body("<tr><th>Activity</th><td>");
    html.push_body(html.http_user.user.activity().await.label());
    html.push_body("</td></tr>");

    html.push_body("<tr><th>Promotion</th><td>");
    html.push_body(html.http_user.user.promotion().await.label());
    html.push_body("</td></tr>");

    html.push_body("<tr><th>Last Lap</th><td>");
    html.push_body(&html.http_user.user.last_lap().await.html_label_full());
    html.push_body("</td></tr>");

    html.push_body("<tr><th>Last Login</th><td>");
    html.push_body(&html.http_user.user.last_login().await.html_label_full());
    html.push_body("</td></tr>");

    html.push_body("<tr><th>Email(s)</th><td>");
    for eml in tbl_eml.items_by_user(&html.http_user.user).await {
        html.push_body(&eml.email().await);
        html.push_body("<br>");
    }
    html.push_body("</td></tr>");

    html.push_body("<tr><th>Password</th><td><div class=\"LiveInput\" id=\"ProfileUserPassword\">");
    html.push_body("<input type=\"password\" name=\"PasswordCurrent\" placeholder=\"current password\"><br>");
    html.push_body("<input type=\"password\" name=\"PasswordNew1\" placeholder=\"new password\"><br>");
    html.push_body("<input type=\"password\" name=\"PasswordNew2\" placeholder=\"repeat password\">");
    html.push_body("<button title=\"Save\">&#128190;</button></div></td></tr>");

    html.push_body("</table></div>");

    Ok(html.into_response().await)
}

pub async fn handler_credentials(State(_app_state): State<AppState>,
                              HttpUserExtractor(http_user): HttpUserExtractor) -> Result<Response, StatusCode> {

    if !http_user.is_logged_in() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let mut html = HtmlTemplate::new(http_user);
    html.include_js("/rsc/js/user.js");

    // change form
    html.push_body("<div class=\"BgBox\"><form acton=\"\" method=\"POST\" class=\"GridForm\" id=\"UserSettingsForm\">");

    // name
    html.push_body(&format!("<label>Name:</label><input type=\"text\" placeholder=\"name\" name=\"new_name\" value=\"{}\">",
                                html.http_user().user.name().await),
    );

    // password
    html.push_body("<label>Current Password:</label><input type=\"password\" placeholder=\"current password\" name=\"old_password\">");
    html.push_body("<label>New Password:</label><input type=\"password\" placeholder=\"new password\" name=\"new_password1\">");
    html.push_body("<label>Verify Password:</label><input type=\"password\" placeholder=\"verify password\" name=\"new_password2\">");

    // email
    html.push_body("<label>Email:</label><input type=\"email\" placeholder=\"email\" name=\"email\" value=\"");
    // if let Some(email) = html.http_user.user.email().await {
    //     html.push_body(&email);
    // }
    html.push_body("\">");

    // save
    html.push_body("<label></label><button type=\"submit\">Save</button>");
    html.push_body("</form></div>");

    Ok(html.into_response().await)
}
