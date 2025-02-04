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
        html.push_body("<div class=\"NoBr\">");
        html.push_body(&eml.email().await);
        html.push_body(" <small>(verified at ");
        html.push_body(&eml.token_verification().await.html_label_full());
        html.push_body(")</small></div><br>");
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

pub async fn handler_accounts(State(app_state): State<AppState>,
                              HttpUserExtractor(http_user): HttpUserExtractor) -> Result<Response, StatusCode> {

    if !http_user.is_logged_in() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // get tables
    let tbl_eml = app_state.database.db_members().await.tbl_email_accounts().await;

    let mut html = HtmlTemplate::new(http_user);
    html.include_css("/rsc/css/user.css");
    html.include_js("/rsc/js/user_accounts.js");

    // Tab Selection
    html.push_body("<div id=\"TabSelection\" class=\"BgBox\">");
    html.push_body("<div>Choose Account Type:</div>");
    html.push_body("<button id=\"AccountTypeButtonPassword\" onclick=\"tabSelectByIndex(0)\" class=\"ActiveButton\">Password</button>");
    html.push_body("<button id=\"AccountTypeButtonEmail\" onclick=\"tabSelectByIndex(1)\">Email</button>");
    html.push_body("<button id=\"AccountTypeButtonSteam\" onclick=\"tabSelectByIndex(2)\">Steam</button>");
    html.push_body("<button id=\"AccountTypeButtonDiscord\" onclick=\"tabSelectByIndex(3)\">Discord</button>");
    html.push_body("</div>");

    // Tab password
    html.push_body("<table id=\"AccountTabPassword\" class=\"LiveInput TabActive BgBox\">");
    html.push_body("<tr><th>Current Password</th><td><input type=\"password\" name=\"PasswordCurrent\" placeholder=\"current password\"></td></tr>");
    html.push_body("<tr><th>New Password</th><td><input type=\"password\" name=\"PasswordNew1\" placeholder=\"new password\"></td></tr>");
    html.push_body("<tr><th>Repeat Password</th><td><input type=\"password\" name=\"PasswordNew2\" placeholder=\"repeat password\"></td></tr>");
    html.push_body("<tr><th></th><td><button title=\"Save\">&#128190;</button>");
    html.push_body("</table>");

    // Tab Email
    html.push_body("<div id=\"AccountTabEmail\" class=\"TabInActive BgBox\">");
    html.push_body("<table>");
    html.push_body("<tr><th>Email</th><th>Verified At</th></tr>");
    for eml in tbl_eml.items_by_user(&html.http_user.user).await.iter() {
        html.push_body("<tr><td>");
        html.push_body(&eml.email().await);
        html.push_body("</td><td>");
        html.push_body(&eml.token_verification().await.html_label_full());
        html.push_body("</tr>");
    }
    html.push_body("</table>");
    html.push_body("</div>");

    // Tab Steam
    html.push_body("<div id=\"AccountTabSteam\" class=\"TabInActive BgBox\">");
    html.push_body("Steam");
    html.push_body("</div>");

    // Tab Discord
    html.push_body("<div id=\"AccountTabDiscord\" class=\"TabInActive BgBox\">");
    html.push_body("Discord");
    html.push_body("</div>");


    Ok(html.into_response().await)
}
