use lettre::{AsyncTransport, Tokio1Executor};
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::{Credentials, Mechanism};
use lettre::transport::smtp::{AsyncSmtpTransport, PoolConfig};
use crate::config::Config;

pub async fn send_email(cfg: &Config, receiver: &str, subject: &str, message: String) -> Result<(), Box<dyn std::error::Error>> {

    // compose email
    let email = lettre::Message::builder()
        .from(Mailbox::new(Some("SSLO League".to_string()), cfg.smtp.email.parse()?))
        .to(receiver.parse()?)
        .subject(subject)
        .body(message).or_else(|e| {
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
