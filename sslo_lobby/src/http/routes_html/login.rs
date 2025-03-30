use axum::extract::{OriginalUri, State};
use axum::http;
use axum::http::header::{REFRESH, SET_COOKIE};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use crate::http::HtmlTemplate;
use crate::app_state::AppState;
use crate::db::members::users::UserItem;
use crate::db::members::steam_accounts::SteamAccountItem;
use crate::http::http_user::HttpUserExtractor;

pub async fn handler(HttpUserExtractor(http_user): HttpUserExtractor,
                     OriginalUri(uri): OriginalUri,
) -> Result<Response, StatusCode> {
    let mut html = HtmlTemplate::new(http_user);
    html.include_css("/rsc/css/lobby/login.css");

    // ensure not logged in
    if html.http_user().is_logged_in() {
        html.message_warning("User already logged in!".to_string());
        return Ok(html.into_response().await);
    }

    if let Some(uri_scheme) = uri.scheme() {
        html.push_body("<div id=\"SteamLogin\">Login via Steam:<br>");
        if let Some(uri_authority) = uri.authority() {
            let steam_url = format!("https://steamcommunity.com/openid/login\
                                     ?openid.ns=http://specs.openid.net/auth/2.0\
                                     &openid.identity=http://specs.openid.net/auth/2.0/identifier_select\
                                     &openid.claimed_id=http://specs.openid.net/auth/2.0/identifier_select\
                                     &openid.mode=checkid_setup\
                                     &openid.return_to={}://{}/html/login/steam",
                                    uri_scheme, uri_authority);
            html.push_body("<a href=\"");
            html.push_body(&steam_url);
            html.push_body("\"><img src=\"https://community.akamai.steamstatic.com/public/images/signinthroughsteam/sits_01.png\"></a>");
            html.push_body("</div>");
        } else {
            log::warn!("Could not extract URI authority from: {}", uri);
            html.message_error("Steam Login Unavailable".to_string());
        }
    } else {
        log::warn!("Could not extract URI scheme from: {}", uri);
        html.message_error("Steam Login Unavailable".to_string());
    }

    Ok(html.into_response().await)
}

pub async fn get_steam_account(app_state: AppState, uri: http::uri::Uri) -> Option<SteamAccountItem> {
    let mut steam_account : Option<SteamAccountItem> = None;
    let db_members = app_state.database.db_members().await;
    let tbl_steam = db_members.tbl_steam_accounts().await;

    if let Some(query) = uri.query() {
        let openid_string = format!("?{}", query);
        if let Ok(params) = steamopenid::kv::decode_keyvalues(&openid_string) {

            // get steam-id
            let steam_id : Option<String> = match params.get("openid.claimed_id") {
                None => None,
                Some(claimed_id) => {
                    let re = regex::Regex::new(r"^https://steamcommunity.com/openid/id/([0-9]+)$").unwrap();
                    match re.captures(claimed_id) {
                        Some(x) => match x.get(1) {
                            Some(y) => Some(y.as_str().to_string()),
                            None => None,
                        },
                        None => None,
                    }
                },
            };

            // verify
            match steamopenid::verify_auth_keyvalues(&params).await {
                Ok(true) => {
                    if let Some(some_steam_id) = steam_id {
                        steam_account = tbl_steam.item_by_steam_id(&some_steam_id, true).await;
                    }
                },
                Ok(false) => {
                    if let Some(some_steam_id) = steam_id {
                        log::warn!("Steam verification failed for ID {}", some_steam_id);
                    }
                }
                Err(e) => {
                    log::error!("Could not verify steam openid parameters {}", e);
                }
            }
        }
    }

    steam_account
}

/// Verify and login from steam authentication
/// When no steam account exists, a new steam account and new user is created
pub async fn handler_steam(HttpUserExtractor(http_user): HttpUserExtractor,
                           State(app_state): State<AppState>,
                           OriginalUri(uri): OriginalUri,
) -> Result<Response, StatusCode> {
    let mut html = HtmlTemplate::new(http_user);

    let db_members = app_state.database.db_members().await;
    let tbl_cookie = db_members.tbl_cookie_logins().await;

    // get user
    let mut user : Option<UserItem> = None;
    if let Some(some_steam_account) = get_steam_account(app_state, uri).await {
        user = some_steam_account.user().await;  // if no user is assigned, a new user is created
    }

    // create login cookie
    let mut cookie: Option<String> = None;
    if let Some(some_user) = user.as_ref() {
        if let Some(login_cookie_item) = tbl_cookie.create_new_cookie(some_user).await {
            cookie = login_cookie_item.get_cookie().await;
        }
    }

    // user info
    if cookie.is_none() {
        html.message_error("Login failed!".to_string());
    }

    // done
    let mut response = html.into_response().await;
    if let Some(cookie) = cookie {
        response.headers_mut().insert(SET_COOKIE, cookie.parse().unwrap());
        response.headers_mut().insert(REFRESH, "0; url=/".parse().unwrap());
    }
    Ok(response)
}
