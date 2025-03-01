use axum::extract::{OriginalUri, Path, State};
use axum::http;
use axum::http::header::{REFRESH, SET_COOKIE};
use axum::http::StatusCode;
use axum::response::Response;
use rand::RngCore;
use crate::app_state::AppState;
use crate::db2::members::email_accounts::EmailAccountItem;
use crate::db2::members::steam_accounts::SteamAccountItem;
use crate::db2::members::users::UserItem;
use crate::http::HtmlTemplate;
use crate::http::http_user::HttpUserExtractor;

pub async fn handler(HttpUserExtractor(http_user): HttpUserExtractor,
                     OriginalUri(uri): OriginalUri,
) -> Result<Response, StatusCode> {

    let mut html = HtmlTemplate::new(http_user);
    html.include_css("/rsc/css/login2.css");
    html.include_js("/rsc/js/login2.js");
    html.push_body("<div class=\"BgBox\">");

    // login/register switch
    html.push_body("<div>");
    html.push_body("<label id=\"LabelLogin\">Login Existing Account</label>");
    html.push_body("<label id=\"SwitchLoginRegister\">");
    html.push_body("<input type=\"checkbox\" />");
    html.push_body("<span></span>");
    html.push_body("</label>");
    html.push_body("<label id=\"LabelRegister\">Create New Account</label>");
    html.push_body("</div>");

    // password
    html.push_body("<div class=\"HrLine\">with Password</div>");
    html.push_body("<input id=\"WithPasswordId\" type=\"text\" placeholder=\"Email or User-ID\"/>");
    html.push_body("<input id=\"WithPasswordPassword\" type=\"password\" placeholder=\"Password\"/>");
    html.push_body("<button id=\"WithPasswordButton\" type=\"button\">Login with Password</button>");

    // email
    html.push_body("<div class=\"HrLine\">with Email</div>");
    html.push_body("<input id=\"WithEmailEmail\" type=\"email\" placeholder=\"Email\"/>");
    html.push_body("<button id=\"WithEmailButton\" type=\"button\">mail Login Link</button>");

    // steam
    html.push_body("<div class=\"HrLine\">with Steam</div>");
    if let Some(uri_scheme) = uri.scheme() {
        if let Some(uri_authority) = uri.authority() {
            let steam_return_url = format!("{}://{}/html/login_steam_verify/",
                                           uri_scheme,
                                           uri_authority);
            html.push_body("<input type=\"hidden\" id=\"WithSteamReturnUrl\" value=\"");
            html.push_body(&steam_return_url);
            html.push_body("\">");
            html.push_body("<button id=\"WithSteamButton\" type=\"button\">Login via Steam Account</button>");
        } else {
            log::warn!("Could not extract URI authority from: {}", uri);
            html.push_body("<span>Steam Login Unavailable</span>");
        }
    } else {
        log::warn!("Could not extract URI scheme from: {}", uri);
        html.push_body("<span>Steam Login Unavailable</span>");
    }

    html.push_body("</div>");
    Ok(html.into_response().await)
}

pub async fn handler_email_create(State(app_state): State<AppState>,
                                  HttpUserExtractor(http_user): HttpUserExtractor,
                                  OriginalUri(uri): OriginalUri,
                                  Path(email): Path<String>,
) -> Result<Response, StatusCode> {
    let mut html = HtmlTemplate::new(http_user);

    // artificial slowdown
    let wait_ms: u64 = 1000u64 + u64::from(rand::thread_rng().next_u32()) / 0x200_000u64; // results in 1000..3048ms
    tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;

    // get db table
    let tbl_eml = app_state.database.db_members().await.tbl_email_accounts().await;

    // get email account
    let mut email_item: EmailAccountItem = match tbl_eml.item_by_email_ignore_verification(&email).await {
        Some(eml) => eml,
        None => match tbl_eml.create_account(email.clone()).await {
            Some(eml) => eml,
            Noone => {
                log::error!("Failed to create new email account with email='{}'", email);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            },
        },
    };

    // create a new token, but only if user is not already existing
    let mut token : Option<String> = None;  // need this option, because build fails when nesting new_email_login_token() and send_email()
    if email_item.has_user().await {
        log::warn!("Deny creating new email account with email='{}', because user already exists.", &email);
    } else {
        token = email_item.create_token().await;
    }

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
                if let Err(e) = crate::helpers::send_email(&app_state.config, &email, "Email Login", &message).await {
                    log::warn!("Could not create new email token for '{}': {}", &email, e)
                }
            } else {
                log::error!("Failed to parse uri.authority() for '{}'", uri.to_string());
            }
        } else {
            log::error!("Failed to parse uri.scheme() for '{}'", uri.to_string());
        }
    }

    // done
    html.message_success("An email with a temporary login link was sent<br><small>(No login link is sent if previous link is still active, or email is invalid)</small>".to_string());
    Ok(html.into_response().await)
}

pub async fn handler_email_existing(State(app_state): State<AppState>,
                                    HttpUserExtractor(http_user): HttpUserExtractor,
                                    OriginalUri(uri): OriginalUri,
                                    Path(email): Path<String>,
) -> Result<Response, StatusCode> {
    let mut html = HtmlTemplate::new(http_user);

    // artificial slowdown
    let wait_ms: u64 = 1000u64 + u64::from(rand::thread_rng().next_u32()) / 0x200_000u64; // results in 1000..3048ms
    tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;

    // get db table
    let tbl_eml = app_state.database.db_members().await.tbl_email_accounts().await;

    // get email account
    if let Some(email_item) = tbl_eml.item_by_email_ignore_verification(&email).await {

        // create a new token, but only if user is not already existing
        let mut token: Option<String> = None;  // need this option, because build fails when nesting new_email_login_token() and send_email()
        if !email_item.has_user().await || email_item.user().await.is_none() {
            log::warn!("Deny logging into existing email account with email='{}', because user does not exists.", &email);
        } else {
            token = email_item.create_token().await;
        }

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
                    if let Err(e) = crate::helpers::send_email(&app_state.config, &email, "Email Login", &message).await {
                        log::warn!("Could not create new email token for '{}': {}", &email, e)
                    }
                } else {
                    log::error!("Failed to parse uri.authority() for '{}'", uri.to_string());
                }
            } else {
                log::error!("Failed to parse uri.scheme() for '{}'", uri.to_string());
            }
        }
    }

    // done
    html.message_success("An email with a temporary login link was sent<br><small>(No login link is sent if previous link is still active, or email is invalid)</small>".to_string());
    Ok(html.into_response().await)
}

pub async fn handler_email_verify(State(app_state): State<AppState>,
                                  HttpUserExtractor(http_user): HttpUserExtractor,
                                  Path((email_account_id_str,token)): Path<(String, String)>,
) -> Result<Response, StatusCode> {
    let mut html = HtmlTemplate::new(http_user);

    // artificial slowdown
    let wait_ms: u64 = 1000u64 + u64::from(rand::thread_rng().next_u32()) / 0x200_000u64; // results in 1000..3048ms
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
        response.headers_mut().insert(REFRESH, "0; url=/".parse().unwrap());
    }
    Ok(response)
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

pub async fn handler_steam_create(State(app_state): State<AppState>,
                                  HttpUserExtractor(http_user): HttpUserExtractor,
                                  OriginalUri(uri): OriginalUri,
) -> Result<Response, StatusCode> {
    let mut html = HtmlTemplate::new(http_user);

    let db_members = app_state.database.db_members().await;
    let tbl_cookie = db_members.tbl_cookie_logins().await;

    // get user
    let mut user : Option<UserItem> = None;
    if let Some(some_steam_account) = get_steam_account(app_state, uri).await {
        if some_steam_account.has_user().await {
            log::warn!("Deny creating new steam account with SteamID='{}', because user already exists.", &some_steam_account.steam_id().await);
        } else {
            user = some_steam_account.user().await;  // this generates a new user and returns it
            if user.is_none() {
                log::error!("Could not create new user for SteamID={}", &some_steam_account.steam_id().await);
                html.message_error("Could not create new user!".to_string());
            }
        }
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

pub async fn handler_steam_existing(State(app_state): State<AppState>,
                                    HttpUserExtractor(http_user): HttpUserExtractor,
                                    OriginalUri(uri): OriginalUri,
) -> Result<Response, StatusCode> {
    let mut html = HtmlTemplate::new(http_user);

    let db_members = app_state.database.db_members().await;
    let tbl_cookie = db_members.tbl_cookie_logins().await;

    // get user
    let mut user : Option<UserItem> = None;
    if let Some(steam_account) = get_steam_account(app_state, uri).await {
        if !steam_account.has_user().await || steam_account.user().await.is_none() {
            log::warn!("Deny logging into existing steam account with SteamID='{}', because user does not exists.", &steam_account.steam_id().await);
        } else {
            user = steam_account.user().await;  // this generates a new user and returns it
            if user.is_none() {
                log::error!("Could not create new user for SteamID={}", &steam_account.steam_id().await);
                html.message_error("Could not create new user!".to_string());
            }
        }
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
