use clap::Parser;
use env_logger::fmt::Formatter;
use std::io::Write;
use std::net::ToSocketAddrs;
use log::{Level, Record};
use app_state::AppState;

mod http;
mod config;
mod app_state;
mod db;

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
async fn main() {
    let cli_args = CliArgs::parse();

    // initialize logging
    env_logger::Builder::new()
        .filter_module("sslo_lib", log::LevelFilter::Debug)
        .filter_module("sslo_lobby", log::LevelFilter::Debug)
        .filter_module("sqlx_core", log::LevelFilter::Warn)
        .format(env_logger_format)
        .init();

    // create app state
    let mut app_state: AppState = match AppState::new(&cli_args.config_file).await {
        Ok(x) => x,
        Err(err) => {
            log::error!("Failed to create AppState: {}", err);
            return;
        }
    };

    // user info
    log::info!("initialization complete");

    // HTTP to HTTPS forwarder (background service)
    tokio::spawn(sslo_lib::http::http2https_background_service(app_state.config.http.url_http.clone(),
                                                               app_state.config.http.url_https.clone()));

    // run https server
    let app = http::create_router(app_state.clone());
    let socket_addr = app_state.config.http.url_https.to_socket_addrs().unwrap().next().unwrap();
    let tls_cfg = app_state.get_rustls_config().await;
    log::info!("starting HTTPS server on {}", socket_addr);
    match axum_server::bind_rustls(socket_addr, tls_cfg).serve(app.into_make_service()).await {
        Ok(_) => {
            log::info!("HTTPS server stopped");
        },
        Err(err) => {
            log::error!("Failed to bind axum server: {}", err);
            return;
        }
    };
}
