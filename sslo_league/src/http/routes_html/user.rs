pub mod accounts;

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
