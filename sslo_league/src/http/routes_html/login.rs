use axum::extract::{OriginalUri, Path, State};
use axum::http::header::{SET_COOKIE, REFRESH};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use rand::RngCore;
use serde::Deserialize;
use crate::app_state::AppState;
use crate::http::HtmlTemplate;
use super::super::http_user::HttpUserExtractor;


pub async fn handler(HttpUserExtractor(http_user): HttpUserExtractor) -> Result<impl IntoResponse, StatusCode> {

    let mut html = HtmlTemplate::new(http_user);
    html.include_css("/rsc/css/login.css");
    html.include_js("/rsc/js/login.js");

    // Tab Selection
    html.push_body("<div id=\"TabSelection\">");
    html.push_body("<div>Choose Login Method:</div>");
    html.push_body("<button id=\"LoginButtonLoginPassword\" onclick=\"tabSelectByIndex(0)\" class=\"ActiveButton\">Password</button>");
    html.push_body("<button id=\"LoginButtonLoginEmail\" onclick=\"tabSelectByIndex(1)\">Email</button>");
    html.push_body("<button id=\"LoginButtonLoginSteam\" onclick=\"tabSelectByIndex(2)\">Steam</button>");
    html.push_body("</div>");

    // Login with Password
    html.push_body("<form id=\"TabLoginPassword\" class=\"ActiveTab\">");
    html.push_body("<label>Login with SSLO Password</label>");
    html.push_body("<input required autofocus placeholder=\"email\" type=\"email\" name=\"LoginEmail\">");
    html.push_body("<input required placeholder=\"password\" type=\"password\" name=\"LoginPassword\">");
    html.push_body("<button type=\"button\">Login</button>");
    html.push_body("</form>");

    // Login with Email SSO
    html.push_body("<form id=\"TabLoginEmail\" action=\"/html/login_email_generate\" method=\"post\">");
    html.push_body("<label>Login via sending Email login link</label>");
    html.push_body("<input required autofocus placeholder=\"email\" type=\"email\" name=\"email\">");
    html.push_body("<button type=\"submit\">Send Login Link</button>");
    html.push_body("</form>");

    // Login with Steam SSO
    html.push_body("<form id=\"TabLoginSteam\">");
    html.push_body("<label>Login via Steam</label>");
    html.push_body("<button type=\"button\">Forward to Steam Login</button>");
    html.push_body("</form>");

    return Ok(html);
}


#[derive(Deserialize)]
pub struct LoginEmailRequestData {
    email: String,
}


pub async fn handler_email_generate(State(app_state): State<AppState>,
                                    HttpUserExtractor(http_user): HttpUserExtractor,
                                    OriginalUri(uri): OriginalUri,
                                    axum::Form(form): axum::Form<LoginEmailRequestData>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut html = HtmlTemplate::new(http_user);
    html.include_css("/rsc/css/login.css");
    html.include_js("/rsc/js/login.js");

    // artificial slowdown
    let wait_ms: u64 = 1000u64 + u64::from(rand::thread_rng().next_u32()) / 0x200_000u64; // should result in ~2000 maximum
    tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;

    // get user
    let mut user_item = app_state.db_members.user_from_email(&form.email).await;
    if user_item.is_none() {  // create new user from email
        user_item = app_state.db_members.user_new_from_email(&form.email).await;
    }

    // create new token
    let mut token : Option<String> = None;  // need this option, because build fails when nesting new_email_login_token() and send_email()
    if let Some(mut some_user_item) = user_item {
        token = some_user_item.update_email_login_token().await;
    }

    // send info email
    if let Some(t) = token {
        if let Some(uri_scheme) = uri.scheme() {
            if let Some(uri_authority) = uri.authority() {
                let link = format!("{}://{}/html/login_email_verify/{}/{}",
                                   uri_scheme,
                                   uri_authority,
                                   &form.email,
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
    Ok(html)
}


pub async fn handler_email_verify(State(app_state): State<AppState>,
                                  HttpUserExtractor(http_user): HttpUserExtractor,
                                  Path((email,token)): Path<(String, String)>,
) -> Result<Response, StatusCode> {
    let mut html = HtmlTemplate::new(http_user);
    html.include_css("/rsc/css/login.css");
    html.include_js("/rsc/js/login.js");

    // artificial slowdown
    let wait_ms: u64 = 1000u64 + u64::from(rand::thread_rng().next_u32()) / 0x200_000u64; // should result in ~2000 maximum
    tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;

    // verify login
    let user_id: Option<i64>;
    let user_item = app_state.db_members.user_from_email(&email).await;
    if let Some(mut some_user_item) = user_item {
        if some_user_item.redeem_email_token(token).await {
            user_id = Some(some_user_item.rowid());
            log::info!("Login with email, user {}:{}", some_user_item.rowid(), &email);
            html.message_success("Successfully logged in.".to_string());
        } else {
            html.message_error("Login failed!".to_string());
            user_id = None;
        }
    } else {
        log::info!("Deny login with invalid email '{}'", email);
        html.message_error("Login failed!".to_string());
        user_id = None;
    }

    // prepare cookie
    let cookie: Option<String> = match user_id {
        None => None,
        Some(id) => {
            Some(app_state.db_members.tbl_cookie_logins.new_cookie(id).await.or_else(|e| {
                log::error!("Failed to create login cookie: {}", e);
                html.message_error("Internal Server Error!".to_string());
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            })?)
        },
    };

    // done
    let mut response = html.into_response();
    if let Some(cookie) = cookie {
        response.headers_mut().insert(SET_COOKIE, cookie.parse().unwrap());
        response.headers_mut().insert(REFRESH, "2; url=/".parse().unwrap());
    }
    Ok(response)
}


pub async fn handler_logout(State(app_state): State<AppState>,
                     HttpUserExtractor(http_user): HttpUserExtractor) -> Result<Response, StatusCode> {

    // deny when not logged in
    if http_user.user_item.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // invalidate token
    let mut cookie_value: Option<String> = None;
    if let Some(item) = http_user.cookie_login_item.as_ref() {
        match app_state.db_members.tbl_cookie_logins.delete_cookie(item).await {
            Ok(cookie) => {
                cookie_value = Some(cookie);
            },
            Err(e) => {
                log::error!("Failed to delete cookie login item[rowid={}]: {:?}", item.rowid, e)
            },
        };
    };

    // generate html
    let name = http_user.name().to_string();
    let mut html = HtmlTemplate::new(http_user);
    html.message_success(format!("Logged out '{}' ...", name));

    // create response
    let mut response = html.into_response();
    if let Some(cookie_value) = cookie_value {
        response.headers_mut().insert(SET_COOKIE, cookie_value.parse().unwrap());
        response.headers_mut().insert(REFRESH, "2; url=/".parse().unwrap());
    }
    Ok(response)
}
