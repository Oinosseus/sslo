use axum::extract::{OriginalUri, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use rand::RngCore;
use serde::Deserialize;
use crate::app_state::AppState;
use crate::http::HtmlTemplate;

#[derive(Deserialize)]
pub struct RegisterSsloFormData {
    login_email: String,
}

pub async fn handler() -> Result<impl IntoResponse, StatusCode> {

    let mut html = HtmlTemplate::new();
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
                                    OriginalUri(uri): OriginalUri,
                                    axum::Form(form): axum::Form<LoginEmailRequestData>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut html = HtmlTemplate::new();
    html.include_css("/rsc/css/login.css");
    html.include_js("/rsc/js/login.js");

    // artificial slowdown
    let wait_ms: u64 = 1000u64 + u64::from(rand::thread_rng().next_u32()) / 0x200_000u64; // should result in ~2000 maximum
    tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;

    // create new token
    let mut token : Option<String> = None;  // need this option, because build fails when nesting new_email_login_token() and send_email()
    if let Ok(t) = app_state.db_members.tbl_emails.new_email_login_token(&form.email).await {
        token = Some(t);
    } else {
        log::warn!("Could not generate email login token for {}", form.email);
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
    html.message_success("An email with a temporary login link was sent.\nNo login link is sent if previous link is still active.".to_string());
    return Ok(html);
}


pub async fn handler_email_verify(State(app_state): State<AppState>,
                                  Path((email,token)): Path<(String, String)>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut html = HtmlTemplate::new();
    html.include_css("/rsc/css/login.css");
    html.include_js("/rsc/js/login.js");

    // artificial slowdown
    let wait_ms: u64 = 1000u64 + u64::from(rand::thread_rng().next_u32()) / 0x200_000u64; // should result in ~2000 maximum
    tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;

    // verify login
    match app_state.db_members.tbl_emails.login_from_email_token(email, token).await {
        Ok(row_user) => {
            html.message_success("Succesfully logged in.".to_string());
        },
        Err(e) => {
            log::warn!("Could not login by email: {}", e);
            html.message_error("Login failed!".to_string());
        }
    }

    // done
    Ok(html)
}
