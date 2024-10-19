use axum::extract::{Host, OriginalUri, State};
use axum::Form;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;
use crate::app_state::AppState;
use crate::db::Database;
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
    html.push_body("<button id=\"RegisterButton\" onclick=\"tabSelectByIndex(3)\">Register</button>");
    html.push_body("</div>");

    // Login with Password
    html.push_body("<form id=\"TabLoginPassword\" class=\"ActiveTab\" method=\"post\">");
    html.push_body("<label>Login with SSLO Password</label>");
    html.push_body("<input required autofocus placeholder=\"email\" type=\"email\" name=\"LoginEmail\">");
    html.push_body("<input required placeholder=\"password\" type=\"password\" name=\"LoginPassword\">");
    html.push_body("<button type=\"submit\">Login</button>");
    html.push_body("</form>");

    // Login with Email SSO
    html.push_body("<form id=\"TabLoginEmail\" method=\"post\">");
    html.push_body("<label>Login via sending Email login link</label>");
    html.push_body("<input required autofocus placeholder=\"email\" type=\"email\" name=\"LoginEmail\">");
    html.push_body("<button type=\"submit\">Send Login Link</button>");
    html.push_body("</form>");

    // Login with Steam SSO
    html.push_body("<form id=\"TabLoginSteam\" method=\"post\">");
    html.push_body("<label>Login via Steam</label>");
    html.push_body("<button type=\"submit\">Forward to Steam Login</button>");
    html.push_body("</form>");

    // SSLO Local Registration
    html.push_body("<form id=\"TabRegisterSsloForm\" method=\"post\" action=\"login/register\">");
    html.push_body("<label>Register with email address</label>");
    // html.push_body("<input type=\"hidden\" name=\"Action\" value=\"Register\">");
    html.push_body("<input required autofocus placeholder=\"email\" type=\"email\" name=\"login_email\">");
    html.push_body("<button type=\"submit\">Send email verification link</button>");
    html.push_body("</form>");

    return Ok(html);
}

pub async fn handler_register(State(app_state): State<AppState>,
                              OriginalUri(uri): OriginalUri,
                              Form(form_data): Form<RegisterSsloFormData>,
) -> Result<impl IntoResponse, StatusCode> {

    println!("HERE '{}'", &uri);
    
    let mut html = HtmlTemplate::new();
    html.include_css("/rsc/css/login.css");
    html.include_js("/rsc/js/login.js");

    // Check if exist in User table
    let existing_user_count = match sqlx::query("SELECT Id FROM Email WHERE Email = $1 LIMIT 1;")
        .bind(&form_data.login_email)
        .fetch_all(app_state.db_members.pool()).await {
            Ok(vec) => vec.len(),
            Err(e) => {
                log::error!("Failed to request DB.members.Email!");
                log::error!("{}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
    };

    // check if exist in NewEmailUser table
    let existing_registration_count = sqlx::query("SELECT Id, CreationTimestamp FROM NewEmailUser WHERE Email = $1 LIMIT 1;")
        .bind(&form_data.login_email)
        .fetch_all(app_state.db_members.pool()).await
        .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?.len();

    // when email is not in the system, yet
    if existing_user_count > 0 {
        log::warn!("Ignored registration for existing Email='{}'", &form_data.login_email);
    } else if existing_registration_count > 0 {
        log::warn!("Ignored registration for existing NewEmailUser='{}'", &form_data.login_email);
    } else {

    // generate new token
        let token: String = app_state.db_members.new_email_user(&form_data.login_email)
            .await.or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;

        // send new token via email
        let email_message = "Follow this link to finish you registration to the SSLO system: ".to_string();
        crate::helpers::send_email(&app_state.config,
                                   &form_data.login_email,
                                   "SSLO League Registration",
                                   email_message)
            .await
            .or_else(|err| {
                log::warn!("Could not send registration email to '{}'", &form_data.login_email);
                log::warn!("{}", err);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            })?;
    }

    // output user info
    html.message_success("An email with a login link should be send to your specified address.".to_string());
    html.message_warning("In case the email address is already registered, or a link was sent within the last 60 minutes, no email was sent.\n\
                          To protect from spying, the actual email transmission is kept secret.".to_string());
    Ok(html)
}


pub async fn handler_login_email(State(app_state): State<AppState>) -> Result<impl IntoResponse, StatusCode> {
    let mut html = HtmlTemplate::new();
    html.include_css("/rsc/css/login.css");
    html.include_js("/rsc/js/login.js");

    todo!();

    Ok(html)
}
