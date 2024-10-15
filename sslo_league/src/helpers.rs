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
        .body(message)?;

    // prepare SMTP config
    let creds = Credentials::new(cfg.smtp.username.clone(), cfg.smtp.password.clone());
    let pool_cfg = PoolConfig::new()
        .max_size(1);
    let sender = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&cfg.smtp.host)?
        .credentials(creds)
        .authentication(vec![Mechanism::Plain])
        .pool_config(pool_cfg)
        .build();

    // transmit email
    sender.send(email).await?;
    Ok(())
}
