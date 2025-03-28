use std::error::Error;
use lettre::{AsyncTransport, Tokio1Executor};
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::{Credentials, Mechanism};
use lettre::transport::smtp::{AsyncSmtpTransport, PoolConfig};
use crate::config::Config;

pub async fn send_email(cfg: &Config, receiver: &str, subject: &str, message: &str) -> Result<(), Box<dyn Error>> {

    // create message body
    let mut body = String::new();
    body += "<!DDOCTYPE html>";
    body += "<html><body>";
    body += message.replace("\n", "<br>").as_str();
    body += "</body></html>";

    // compose email
    let email = lettre::Message::builder()
        .header(lettre::message::header::ContentType::TEXT_HTML)
        .from(Mailbox::new(Some("SSLO League".to_string()), cfg.smtp.email.parse()?))
        .to(receiver.parse()?)
        .subject(subject)
        .body(body).or_else(|e| {
            log::error!("Could not compose email: {}", e);
            Err(e)
    })?;

    // prepare SMTP config
    let creds = Credentials::new(cfg.smtp.username.clone(), cfg.smtp.password.clone());
    let pool_cfg = PoolConfig::new()
        .max_size(1);
    let smtp_transporter = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&cfg.smtp.host)
        .or_else(|e| {
            log::error!("Could not start SMTP transport: {}", e);
            Err(e)
        })?;
    let sender = smtp_transporter.credentials(creds)
        .authentication(vec![Mechanism::Plain])
        .pool_config(pool_cfg)
        .build();

    // transmit email
    sender.send(email)
        .await
        .or_else(|e| {
            log::warn!("Failed to send message to '{}': {}", receiver, e);
            Err(e)
        })?;
    Ok(())
}

pub fn now() -> chrono::DateTime<chrono::Utc> {
    chrono::offset::Utc::now()
}
