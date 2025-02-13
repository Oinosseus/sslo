use axum::extract::{OriginalUri, Path, State};
use axum::http::header::{SET_COOKIE, REFRESH};
use axum::http::StatusCode;
use axum::response::Response;
use rand::RngCore;
use serde::Deserialize;
use crate::app_state::AppState;
use crate::db2::members::users::UserItem;
use crate::db2::members::email_accounts::EmailAccountItem;
use crate::db2::members::steam_accounts::SteamAccountItem;
use crate::http::HtmlTemplate;
use crate::http::http_user::HttpUser;
use super::super::http_user::HttpUserExtractor;


pub async fn handler(HttpUserExtractor(http_user): HttpUserExtractor,
                     OriginalUri(uri): OriginalUri,
) -> Result<Response, StatusCode> {

    let mut html = HtmlTemplate::new(http_user);
    html.include_css("/rsc/css/login.css");
    html.include_js("/rsc/js/login.js");

    // Tab Selection
    html.push_body("<div id=\"TabSelection\" class=\"BgBox\">");
    html.push_body("<div>Choose Login Method:</div>");
    html.push_body("<button id=\"LoginButtonLoginPassword\" onclick=\"tabSelectByIndex(0)\" class=\"ActiveButton\">Password</button>");
    html.push_body("<button id=\"LoginButtonLoginEmail\" onclick=\"tabSelectByIndex(1)\">Email</button>");
    html.push_body("<button id=\"LoginButtonLoginSteam\" onclick=\"tabSelectByIndex(2)\">Steam</button>");
    html.push_body("</div>");

    // Login with Password
    // html.push_body("<form id=\"TabLoginPassword\" action=\"/html/login_password\" method=\"post\" class=\"ActiveTab BgBox\">");
    html.push_body("<form id=\"TabLoginPassword\" method=\"post\" class=\"ActiveTab BgBox\">");
    html.push_body("<label>Login with SSLO Password</label>");
    html.push_body("<input required autofocus placeholder=\"email or id\" type=\"text\" name=\"email_or_id\">");
    html.push_body("<input required placeholder=\"password\" type=\"password\" name=\"password\">");
    html.push_body("<button type=\"submit\">Login</button>");
    // html.push_body("<button onclick=\"login_with_password\">Login</button>");
    html.push_body("</form>");

    // Login with Email SSO
    html.push_body("<form id=\"TabLoginEmail\" action=\"/html/login_email_generate\" method=\"post\" class=\"BgBox\">");
    html.push_body("<label>Login via sending Email login link</label>");
    html.push_body("<input required autofocus placeholder=\"email\" type=\"email\" name=\"email\">");
    html.push_body("<button type=\"submit\">Send Login Link</button>");
    html.push_body("</form>");

    // Login with Steam SSO
    html.push_body("<form id=\"TabLoginSteam\" class=\"BgBox\">");
    html.push_body("<label>Login via Steam</label>");
    if let Some(uri_scheme) = uri.scheme() {
        if let Some(uri_authority) = uri.authority() {
            let steam_return_link = format!("{}://{}/html/login_steam_verify/",
                                            uri_scheme,
                                            uri_authority);
            html.push_body("<a href=\"");
            html.push_body("https://steamcommunity.com/openid/login");
            html.push_body("?openid.ns=http://specs.openid.net/auth/2.0");
            html.push_body("&openid.identity=http://specs.openid.net/auth/2.0/identifier_select");
            html.push_body("&openid.claimed_id=http://specs.openid.net/auth/2.0/identifier_select");
            html.push_body("&openid.mode=checkid_setup");
            html.push_body("&openid.return_to=");
            html.push_body(&steam_return_link);
            html.push_body("\" target=\"_top\"><img src=\"https://community.fastly.steamstatic.com/public/images/signinthroughsteam/sits_01.png\"></a>");
        } else {
            log::warn!("Could not extract URI authority from: {}", uri);
            html.push_body("<span>Steam Login Unavailable</span>");
        }
    } else {
        log::warn!("COuld not extract URI scheme from: {}", uri);
        html.push_body("<span>Steam Login Unavailable</span>");
    }
    html.push_body("</form>");

    Ok(html.into_response().await)
}


#[derive(Deserialize)]
pub struct LoginEmailRequestData {
    email: String,
    password: Option<String>,
}


pub async fn handler_email_generate(State(app_state): State<AppState>,
                                    HttpUserExtractor(http_user): HttpUserExtractor,
                                    OriginalUri(uri): OriginalUri,
                                    axum::Form(form): axum::Form<LoginEmailRequestData>,
) -> Result<Response, StatusCode> {
    let mut html = HtmlTemplate::new(http_user);
    html.include_css("/rsc/css/login.css");
    html.include_js("/rsc/js/login.js");

    // artificial slowdown
    let wait_ms: u64 = 1000u64 + u64::from(rand::thread_rng().next_u32()) / 0x200_000u64; // should result in ~2000 maximum
    tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;

    // get db table
    let tbl_eml = app_state.database.db_members().await.tbl_email_accounts().await;

    // get email account
    let mut token : Option<String> = None;  // need this option, because build fails when nesting new_email_login_token() and send_email()
    let mut email_item: EmailAccountItem = match tbl_eml.item_by_email_ignore_verification(&form.email).await {
        Some(eml) => eml,
        None => match tbl_eml.create_account(form.email.clone()).await {
            Some(new_eml) => new_eml,
            None => {
                log::error!("Failed to create new email account with email='{}'", form.email);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        },
    };

    // create new token
    token = email_item.create_token().await;

    // send info email
    if let Some(t) = token {
        if let Some(uri_scheme) = uri.scheme() {
            if let Some(uri_authority) = uri.authority() {
                let link = format!("{}://{}/html/login_email_verify/{}/{}",
                                   uri_scheme,
                                   uri_authority,
                                   email_item.id().await,
                                   t);
                let message = format!("Hello User,<br><br>please follow this link to login into the SSLO League: <a href=\"{}\">{}</a>.<br><br>Regards",
                                      link, uri_authority);
                if let Err(e) = crate::helpers::send_email(&app_state.config, &form.email, "Email Login", &message).await {
                    log::warn!("Could not create new email token for '{}': {}", &form.email, e)
                }
            } else {
                log::error!("Failed to parse uri.authority() for '{}'", uri.to_string());
            }
        } else {
            log::error!("Failed to parse uri.scheme() for '{}'", uri.to_string());
        }
    }

    // done
    html.message_success("An email with a temporary login link was sent.".to_string());
    html.message_warning("No login link is sent if previous link is still active, or email is invalid.".to_string());
    Ok(html.into_response().await)
}

pub async fn handler_email_verify(State(app_state): State<AppState>,
                                  HttpUserExtractor(http_user): HttpUserExtractor,
                                  Path((email_account_id_str,token)): Path<(String, String)>,
) -> Result<Response, StatusCode> {
    let mut html = HtmlTemplate::new(http_user);
    html.include_css("/rsc/css/login.css");
    html.include_js("/rsc/js/login.js");

    // artificial slowdown
    let wait_ms: u64 = 1000u64 + u64::from(rand::thread_rng().next_u32()) / 0x200_000u64; // should result in ~2000 maximum
    tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;

    // get db tables
    let tbl_cookie = app_state.database.db_members().await.tbl_cookie_logins().await;
    let tbl_eml = app_state.database.db_members().await.tbl_email_accounts().await;

    // extract email account id
    let email_account_id: i64 = match email_account_id_str.parse() {
        Ok(id) => id,
        Err(_) => {
            log::warn!("Failed to parse email account id from {}", email_account_id_str);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    // verify login
    let mut cookie: Option<String> = None;
    if let Some(eml) = tbl_eml.item_by_id(email_account_id).await {  // get email account
        if eml.consume_token(token).await {  // verify token
            if let Some(user) = eml.user().await {  // get assigned user
                if let Some(login_cookie_item) = tbl_cookie.create_new_cookie(&user).await {
                    cookie = login_cookie_item.get_cookie().await;
                }
            } else {
                log::error!("Could not retrieve user from valid email account {}", eml.display().await);
            }
        }
    } else {
        log::warn!("could not find email account from {}", email_account_id);
    }

    // user info
    if cookie.is_none() {
        html.message_error("Login failed!".to_string());
    }

    // done
    let mut response = html.into_response().await;
    if let Some(cookie) = cookie {
        response.headers_mut().insert(SET_COOKIE, cookie.parse().unwrap());
        response.headers_mut().insert(REFRESH, "1; url=/".parse().unwrap());
    }
    Ok(response)
}


pub async fn handler_logout(State(app_state): State<AppState>,
                     HttpUserExtractor(mut http_user): HttpUserExtractor) -> Result<Response, StatusCode> {

    // get tables
    let tbl_cookie = app_state.database.db_members().await.tbl_cookie_logins().await;
    let name = http_user.user.name().await;

    let mut cookie_value: Option<String> = None;
    if let Some(cookie_login) = http_user.cookie_login.take() {
        cookie_value = Some(tbl_cookie.delete_cookie(cookie_login).await);  // invalidate login cookie
        http_user = HttpUser::new_anonymous(app_state).await;  // downgrade http user
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // generate html
    let mut html = HtmlTemplate::new(http_user);
    html.message_success(format!("Logged out '{}' ...", name));

    // create response
    let mut response = html.into_response().await;
    if let Some(cookie_value) = cookie_value {
        response.headers_mut().insert(SET_COOKIE, cookie_value.parse().unwrap());
        response.headers_mut().insert(REFRESH, "1; url=/".parse().unwrap());
    }
    Ok(response)
}


pub async fn handler_steam_verify(State(app_state): State<AppState>,
                                  HttpUserExtractor(http_user): HttpUserExtractor,
                                  OriginalUri(uri): OriginalUri,
) -> Result<Response, StatusCode> {
    let mut html = HtmlTemplate::new(http_user);
    html.include_css("/rsc/css/login.css");

    let db_members = app_state.database.db_members().await;
    let tbl_steam = db_members.tbl_steam_accounts().await;
    let tbl_users = db_members.tbl_users().await;
    let tbl_cookie = db_members.tbl_cookie_logins().await;

    let mut user : Option<UserItem> = None;
    let mut cookie: Option<String> = None;

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
            let mut steam_account : Option<SteamAccountItem> = None;
            match steamopenid::verify_auth_keyvalues(&params).await {
                Ok(true) => {
                    if let Some(some_steam_id) = steam_id {
                        steam_account = tbl_steam.item_by_steam_id(&some_steam_id, true).await;
                    }
                },
                Ok(false) => {
                    html.message_error("Steam ID verification failed!".to_string());
                }
                Err(e) => {
                    log::error!("Could not verify steam openid parameters {}", e);
                    html.message_error("Could not verify steam openid parameters".to_string());
                }
            }

            // get user
            if let Some(some_steam_account) = steam_account {
                user = some_steam_account.user().await;
                if user.is_none() {
                    user = tbl_users.create_new_user().await;
                    if let Some(some_user) = user.as_ref() {
                        match some_steam_account.set_user(some_user).await {
                            Ok(_) => {},
                            Err(e) => {
                                log::error!("Could not assign steam account to {}: {}",
                                    some_steam_account.display().await, e);
                                html.message_error("Could not assign steam account to {}".to_string());
                            }
                        }
                    } else {
                        html.message_error("Could not create new user!".to_string());
                    }
                }
            }
        }
    }

    // create login cookie
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
        response.headers_mut().insert(REFRESH, "1; url=/".parse().unwrap());
    }
    Ok(response)
}