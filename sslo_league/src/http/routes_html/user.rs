use axum::extract::{OriginalUri, State};
use axum::http::StatusCode;
use axum::response::Response;
use sslo_lib::optional_date::OptionalDateTime;
use crate::app_state::AppState;
use crate::http::HtmlTemplate;
use crate::http::http_user::HttpUserExtractor;


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
        html.push_body("</div><br>");
    }
    html.push_body("</td></tr>");

    html.push_body("</table></div>");

    Ok(html.into_response().await)
}

pub async fn handler_credentials(State(app_state): State<AppState>,
                                 HttpUserExtractor(http_user): HttpUserExtractor) -> Result<Response, StatusCode> {

    if !http_user.is_logged_in() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // get tables
    let db_members = app_state.database.db_members().await;
    let tbl_eml = db_members.tbl_email_accounts().await;
    let tbl_steam = db_members.tbl_steam_accounts().await;

    let mut html = HtmlTemplate::new(http_user);
    html.include_css("/rsc/css/user.css");
    html.include_js("/rsc/js/user_accounts.js");

    // Tab Selection
    html.push_body("<div id=\"TabSelection\" class=\"BgBox\">");
    html.push_body("<div>Choose Account Type:</div>");
    html.push_body("<button id=\"AccountTypeButtonPassword\" onclick=\"tabSelectByIndex(0)\" class=\"ActiveButton\">Password</button>");
    html.push_body("<button id=\"AccountTypeButtonEmail\" onclick=\"tabSelectByIndex(1)\">Emails</button>");
    html.push_body("<button id=\"AccountTypeButtonSteam\" onclick=\"tabSelectByIndex(2)\">Steam</button>");
    html.push_body("<button id=\"AccountTypeButtonDiscord\" onclick=\"tabSelectByIndex(3)\">Discord</button>");
    html.push_body("</div>");

    // Tab password
    html.push_body("<table id=\"AccountTabPassword\" class=\"LiveInput TabActive BgBox\">");
    html.push_body("<tr><th>Current Password</th><td><input type=\"password\" name=\"PasswordCurrent\" placeholder=\"current password\"></td></tr>");
    html.push_body("<tr><th>New Password</th><td><input type=\"password\" name=\"PasswordNew1\" placeholder=\"new password\"></td></tr>");
    html.push_body("<tr><th>Repeat Password</th><td><input type=\"password\" name=\"PasswordNew2\" placeholder=\"repeat password\"></td></tr>");
    html.push_body("<tr><th></th><td><button title=\"Save\">&#128190; Save</button>");
    html.push_body("</table>");

    // Tab Email
    html.push_body("<div id=\"AccountTabEmail\" class=\"TabInActive BgBox\">");
    html.push_body("<table><tr><th>Email</th><th>Verified At</th></tr>");
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
    html.push_body("<table><tr><th>Steam ID</th><th>Created At</th></tr>");
    for steam in tbl_steam.items_by_user(&html.http_user.user).await.iter() {
        html.push_body("<tr><td>");
        html.push_body(&steam.steam_id().await);
        html.push_body("</td><td>");
        let dt = OptionalDateTime::new(Some(steam.creation().await));
        html.push_body(&dt.html_label_full());
        html.push_body("</tr>");
    }
    html.push_body("</table>");
    html.push_body("</div>");

    // Tab Discord
    html.push_body("<div id=\"AccountTabDiscord\" class=\"TabInActive BgBox\">");
    html.push_body("Discord");
    html.push_body("</div>");


    Ok(html.into_response().await)
}
