use clap::Parser;
use axum_server::tls_rustls::RustlsConfig;
use std::net::{Ipv4Addr, SocketAddr};
use app_state::AppState;

mod http;
mod config;
mod app_state;
mod db;

#[derive(Parser)]
struct CliArgs {
    config_file: std::path::PathBuf,
}


#[tokio::main]
async fn main() {

    let cli_args = CliArgs::parse();

    // create app state
    let mut app_state: AppState = AppState::new(&cli_args.config_file).unwrap();
    app_state.init().await.unwrap();

    // initialize logging
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .format_target(true)
        .init();

    // HTTP to HTTPS forwarder (background service)
    tokio::spawn(http::http2https_background_service(app_state.config.http.port_http, app_state.config.http.port_https));

    // user info
    log::info!("initialization complete");

    // run https server
    let app = http::create_router(app_state.clone());
    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, app_state.config.http.port_https));
    let tls_cfg = app_state.get_rustls_config().await;
    axum_server::bind_rustls(addr, tls_cfg)
        .serve(app.into_make_service())
        .await.unwrap()
}
