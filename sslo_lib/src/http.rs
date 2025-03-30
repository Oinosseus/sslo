use std::net::{Ipv4Addr, SocketAddr};
use axum::extract::{Host, Path};
use axum::handler::HandlerWithoutStateExt;
use axum::http::{header, StatusCode, Uri};
use axum::response::{IntoResponse, Redirect};
use rust_embed::RustEmbed;

#[allow(dead_code)]
pub async fn http2https_background_service(port_http: u16, port_https: u16) {
    // Implementation from:
    // https://github.com/tokio-rs/axum/blob/main/examples/tls-rustls/src/main.rs

    fn make_https(host: String, uri: Uri, port_http: u16, port_https: u16) -> Result<Uri, axum::BoxError> {
        let mut parts = uri.into_parts();

        parts.scheme = Some(axum::http::uri::Scheme::HTTPS);

        if parts.path_and_query.is_none() {
            parts.path_and_query = Some("/".parse().unwrap());
        }

        let https_host = host.replace(&port_http.to_string(), &port_https.to_string());
        parts.authority = Some(https_host.parse()?);

        Ok(Uri::from_parts(parts)?)
    }

    let redirect = move |Host(host): Host, uri: Uri| async move {
        match make_https(host, uri, port_http, port_https) {
            Ok(uri) => Ok(Redirect::permanent(&uri.to_string())),
            Err(_) => {
                Err(StatusCode::BAD_REQUEST)
            }
        }
    };

    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, port_http));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, redirect.into_make_service())
        .await
        .unwrap();
}

pub enum FrontendMessage {
    Success(String),
    Warning(String),
    Error(String),
}

impl FrontendMessage {

    pub fn extract_message(&self) -> String {
        match self {
            FrontendMessage::Success(msg) => msg.clone(),
            FrontendMessage::Warning(msg) => msg.clone(),
            FrontendMessage::Error(msg) => msg.clone(),
        }
    }

    pub fn to_html(&self) -> String {
        let mut html = String::new();
        html += match self {
            Self::Success(_) => "<div class=\"MessageSuccess\">",
            Self::Warning(_) => "<div class=\"MessageWarning\">",
            Self::Error(_) => "<div class=\"MessageError\">",
        };
        html += &self.extract_message().replace("\n", "<br>");
        html += "</div>";
        html
    }
}


#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/../rsc"]
struct Resources;

/// axum route handler for static resources in /rsc project directory
///
/// Integrate like: Router::new().route("/rsc/*filepath", routing::get(route_handler_static_resources))
pub async fn route_handler_static_resources(Path(filepath): Path<String>) -> Result<impl IntoResponse, StatusCode> {
    let fileconent = Resources::get(&filepath).ok_or_else(|| StatusCode::NOT_FOUND)?;

    // find content-type
    let mime_type : &'static str;
    if      filepath.ends_with(".css") { mime_type = "text/css" }
    else if filepath.ends_with(".js")  { mime_type = "application/javascript" }
    else if filepath.ends_with(".png") { mime_type = "image/png" }
    else if filepath.ends_with(".svg") { mime_type = "image/svg+xml" }
    else if filepath.ends_with(".ico") { mime_type = "image/x-icon" }
    else {
        return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    // return file
    Ok((StatusCode::OK,
        [(header::CONTENT_TYPE, mime_type)],
        fileconent.data))
}
