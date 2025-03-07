use axum::extract::{OriginalUri, State};
use axum::http::StatusCode;
use axum::response::Response;
use axum::Json;
use axum::response::IntoResponse;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use crate::app_state::AppState;
use crate::db2::members::email_accounts::EmailAccountItem;
use crate::db2::members::users::UserItem;
use crate::http::http_user::HttpUserExtractor;
use crate::http::routes_rest_v0::GeneralError;

#[derive(Deserialize)]
pub struct RequestData {
    email: String,
}

pub async fn email_put(State(app_state): State<AppState>,
                       HttpUserExtractor(http_user): HttpUserExtractor,
                       OriginalUri(uri): OriginalUri,
                       Json(input): Json<RequestData>,
) -> Response {

    // verify user
    if http_user.user.id().await <= 0 {
        log::warn!("Deny adding email '{}' to invalid {}", input.email, http_user.user.display().await);
        return GeneralError::new(StatusCode::FORBIDDEN, "No valid user logged in to add an email account!".to_string()).into_response();
    }

    // artificial slowdown
    let wait_ms: u64 = 1000u64 + u64::from(rand::thread_rng().next_u32()) / 0x200_000u64; // results in 1000..3048ms
    tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;

    // get email account and create a token
    let tbl_eml = app_state.database.db_members().await.tbl_email_accounts().await;
    let email_item: EmailAccountItem = match tbl_eml.item_by_email_ignore_verification(&input.email).await {
        Some(eml) => eml,
        None => match tbl_eml.create_account(input.email.clone()).await {
            Some(eml) => {
                if !eml.set_user(&http_user.user).await {
                    return GeneralError::new(StatusCode::INTERNAL_SERVER_ERROR, "Failed to assign user to email account!".to_string()).into_response()
                }
                eml
            },
            None => {
                log::error!("Failed to create new email account with email='{}'", input.email);
                return GeneralError::new(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create email account!".to_string()).into_response()
            },
        },
    };
    let token : Option<String> = email_item.create_token().await;

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
                if let Err(e) = crate::helpers::send_email(&app_state.config, &input.email, "Email Login", &message).await {
                    log::warn!("Could not create new email token for '{}': {}", &input.email, e);
                    return GeneralError::new(StatusCode::INTERNAL_SERVER_ERROR, "Could not create new email token!".to_string()).into_response()
                }
            } else {
                log::error!("Failed to parse uri.authority() for '{}'", uri.to_string());
                return GeneralError::new(StatusCode::INTERNAL_SERVER_ERROR, "CFailed to parse uri.authority()!".to_string()).into_response()
            }
        } else {
            log::error!("Failed to parse uri.scheme() for '{}'", uri.to_string());
            return GeneralError::new(StatusCode::INTERNAL_SERVER_ERROR, "CFailed to parse uri.scheme()!".to_string()).into_response()
        }
    }

    StatusCode::NO_CONTENT.into_response()
}
