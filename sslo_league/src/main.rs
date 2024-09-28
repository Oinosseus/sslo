use clap::Parser;
use axum_server::tls_rustls::RustlsConfig;
use std::net::{Ipv4Addr, SocketAddr};

mod http;
mod config;

// temporarily until a config is available
static CONFIG_PORT_HTTP: u16 = 8080;
static CONFIG_PORT_HTTPS: u16 = 8443;

#[derive(Parser)]
struct CliArgs {
    config_file: std::path::PathBuf,
}



#[tokio::main]
async fn main() {
    let cli_args = CliArgs::parse();

    // load config
    let _config = match config::Config::from_file(cli_args.config_file) {
        Ok(config) => config,
        Err(e) => {panic!("{e}")}
    };

    // create TLS config
    let tls_cfg = RustlsConfig::from_pem_file(
        "test_db/tls/cert.pem",
        "test_db/tls/key.pem"
    ).await.unwrap();

    // HTTP to HTTPS forwarder (background service)
    tokio::spawn(http::http2https_background_service());

    // run https server
    let app = http::create_router();
    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, CONFIG_PORT_HTTPS));
    axum_server::bind_rustls(addr, tls_cfg)
        .serve(app.into_make_service())
        .await.unwrap()
}
