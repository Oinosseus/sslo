use std::net::{Ipv4Addr, SocketAddr};
use axum_server::tls_rustls::RustlsConfig;

mod http;

// temporarily until a config is available
static CONFIG_PORT_HTTP: u16 = 8080;
static CONFIG_PORT_HTTPS: u16 = 8443;

#[tokio::main]
async fn main() {
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
