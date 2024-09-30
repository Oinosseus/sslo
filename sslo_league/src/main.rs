use clap::Parser;
use axum_server::tls_rustls::RustlsConfig;
use std::net::{Ipv4Addr, SocketAddr};

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

    // initialize logging
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .format_target(true)
        .init();

    let cli_args = CliArgs::parse();

    // load config
    let config : config::Config = config::Config::from_file(cli_args.config_file)
        .unwrap_or_else(|e| { panic!("{e}") });

    // create app state
    let app_state = app_state::AppState::new(&config);

    // create TLS config
    let tls_cfg = RustlsConfig::from_pem_file(
        "test_db/tls/cert.pem",
        "test_db/tls/key.pem"
    ).await.unwrap();

    // HTTP to HTTPS forwarder (background service)
    tokio::spawn(http::http2https_background_service(config.http.port_http, config.http.port_https));

    // run https server
    let app = http::create_router(app_state.clone());
    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, config.http.port_https));
    axum_server::bind_rustls(addr, tls_cfg)
        .serve(app.into_make_service())
        .await.unwrap()
}
