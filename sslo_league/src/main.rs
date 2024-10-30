use std::error::Error;
use clap::Parser;
use std::net::{Ipv4Addr, SocketAddr};
use env_logger::fmt::Formatter;
use std::io::Write;
use log::{error, Level, Record};
use sqlx::error::BoxDynError;
use app_state::AppState;

mod http;
mod config;
mod app_state;
mod db;
mod helpers;
mod user_grade;

#[derive(Parser)]
struct CliArgs {
    config_file: std::path::PathBuf,
}

fn env_logger_format(buf: &mut Formatter, record: &Record<'_>) -> std::io::Result<()> {
    let color: &'static str = match record.level() {
        Level::Error => "\x1b[91m",
        Level::Warn => "\x1b[93m",
        Level::Info => "\x1b[97m",
        Level::Debug => "\x1b[35m",
        Level::Trace => "\x1b[37m",
    };
    writeln!(buf, "\x1b[37m{} {}{} \x1b[3;37m{}:{} {}{}\x1b[0m",
             chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
             color,
             record.level(),
             record.module_path().unwrap_or("unknown"),
             record.line().unwrap_or(0),
             color,
             record.args())?;
    Ok(())
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let cli_args = CliArgs::parse();

    // initialize logging
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .format(env_logger_format)
        .init();

    // create app state
    let mut app_state: AppState = AppState::new(&cli_args.config_file).or_else(|e| {
        error!("Failed to create application state: {}", e);
        Err(e)
    })?;
    app_state.init().await?;

    // user info
    log::info!("initialization complete");

    // HTTP to HTTPS forwarder (background service)
    tokio::spawn(http::http2https_background_service(app_state.config.http.port_http, app_state.config.http.port_https));

    // run https server
    let app = http::create_router(app_state.clone());
    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, app_state.config.http.port_https));
    let tls_cfg = app_state.get_rustls_config().await;
    axum_server::bind_rustls(addr, tls_cfg)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
