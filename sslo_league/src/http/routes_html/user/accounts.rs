use axum::extract::{OriginalUri, State};
use axum::http::StatusCode;
use axum::response::Response;
use sslo_lib::optional_date::OptionalDateTime;
use crate::app_state::AppState;
use crate::http::HtmlTemplate;
use crate::http::http_user::HttpUserExtractor;

pub async fn handler(State(app_state): State<AppState>,
                     HttpUserExtractor(http_user): HttpUserExtractor,
                     OriginalUri(uri): OriginalUri,
) -> Result<Response, StatusCode> {

    if !http_user.is_logged_in() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // get tables
    let db_members = app_state.database.db_members().await;
    let tbl_eml = db_members.tbl_email_accounts().await;
    let tbl_steam = db_members.tbl_steam_accounts().await;

    let mut html = HtmlTemplate::new(http_user);
    html.include_css("/rsc/css/user_accounts.css");
    html.include_js("/rsc/js/user_accounts.js");
    html.push_body("<div class=\"BgBox\">");

    // Tab Selection
    html.push_body("<div id=\"AccountTypeSelection\">");
    html.push_body("<button id=\"AccountTypeButtonPassword\" onclick=\"tabSelectByIndex(0)\" class=\"ActiveButton\">SSLO Password</button>");
    html.push_body("<button id=\"AccountTypeButtonEmail\" onclick=\"tabSelectByIndex(1)\">Emails</button>");
    html.push_body("<button id=\"AccountTypeButtonSteam\" onclick=\"tabSelectByIndex(2)\">Steam</button>");
    html.push_body("<button id=\"AccountTypeButtonDiscord\" onclick=\"tabSelectByIndex(3)\">Discord</button>");
    html.push_body("</div><hr>");

    // Tab password
    html.push_body("<table id=\"AccountTabPassword\" class=\"LiveInput TabActive\">");
    html.push_body("<tr><th>Current Password</th><td><input type=\"password\" name=\"PasswordCurrent\" placeholder=\"current password\"></td></tr>");
    html.push_body("<tr><th>New Password</th><td><input type=\"password\" name=\"PasswordNew1\" placeholder=\"new password\"></td></tr>");
    html.push_body("<tr><th>Repeat Password</th><td><input type=\"password\" name=\"PasswordNew2\" placeholder=\"repeat password\"></td></tr>");
    html.push_body("<tr><th></th><td><button title=\"Save\">&#128190; Save</button>");
    html.push_body("</table>");

    // Tab Email
    html.push_body("<div id=\"AccountTabEmail\" class=\"TabInActive\">");
    html.push_body("<table><tr><th>Email</th><th>Verified At</th></tr>");
    for eml in tbl_eml.items_by_user(&html.http_user.user).await.iter() {
        html.push_body("<tr><td>");
        html.push_body(&eml.email().await);
        html.push_body("</td><td>");
        html.push_body(&eml.token_verification().await.html_label_full());
        html.push_body("</td><td>");
        html.push_body("<button class=\"ButtonDelete\" onclick=\"handler_button_delete_email('");
        html.push_body(&eml.email().await.to_string());
        html.push_body("')\" title=\"remove Steam account\"></button>");
        html.push_body("</td></tr>");
    }
    html.push_body("<tr><td>");
    html.push_body("<input type=\"email\" id=\"AddEmail\" placeholder=\"Additional email\"></td><td></td><td>");
    html.push_body("<button title=\"Mail verification link\" class=\"ButtonAdd\" onclick=\"handler_button_add_email()\"></button>");
    html.push_body("</td></tr>");
    html.push_body("</table>");
    html.push_body("</div>");

    // Tab Steam
    html.push_body("<div id=\"AccountTabSteam\" class=\"TabInActive\">");
    html.push_body("<table><tr><th>Steam ID</th><th>Created At</th></tr>");
    for steam in tbl_steam.items_by_user(&html.http_user.user).await.iter() {
        html.push_body("<tr><td>");
        html.push_body(&steam.steam_id().await);
        html.push_body("</td><td>");
        let dt = OptionalDateTime::new(Some(steam.creation().await));
        html.push_body(&dt.html_label_full());
        html.push_body("</td><td>");
        html.push_body("<button class=\"ButtonDelete\" onclick=\"handler_button_delete_steam(");
        html.push_body(&steam.id().await.to_string());
        html.push_body(")\" title=\"remove Steam account\"></button>");
        html.push_body("</td></tr>");
    }
    html.push_body("</table>");
    if let Some(uri_scheme) = uri.scheme() {
        if let Some(uri_authority) = uri.authority() {
            html.push_body("<a href=\"");
            html.push_body("https://steamcommunity.com/openid/login");
            html.push_body("?openid.ns=http://specs.openid.net/auth/2.0");
            html.push_body("&openid.identity=http://specs.openid.net/auth/2.0/identifier_select");
            html.push_body("&openid.claimed_id=http://specs.openid.net/auth/2.0/identifier_select");
            html.push_body("&openid.mode=checkid_setup");
            html.push_body("&openid.return_to=");
            html.push_body(&format!("{}://{}/html/login_steam_assign", uri_scheme, uri_authority));
            html.push_body("\" target=\"_top\"><img src=\"https://community.fastly.steamstatic.com/public/images/signinthroughsteam/sits_01.png\"></a>");
        } else {
            log::warn!("Could not extract URI authority from: {}", uri);
        }
    } else {
        log::warn!("Could not extract URI scheme from: {}", uri);
    }
    html.push_body("</div>");

    // Tab Discord
    html.push_body("<div id=\"AccountTabDiscord\" class=\"TabInActive\">");
    html.push_body("Discord");
    html.push_body("</div>");


    html.push_body("</div>");
    Ok(html.into_response().await)
}
