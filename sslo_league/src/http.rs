use std::net::{Ipv4Addr, SocketAddr};
use axum::extract::Host;
use axum::handler::HandlerWithoutStateExt;
use axum::http::{StatusCode, Uri};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::{routing, Router};
use sslo_lib::http_routes::static_resources;
use crate::app_state::AppState;

mod routes_html;
mod routes_rest_v0;
mod extract_user;

struct HtmlTemplate {
    html_body: String,
    css_files: Vec<& 'static str>,
    js_files: Vec<& 'static str>,
    frontend_messages: Vec<FrontendMessage>,
}

impl HtmlTemplate {

    pub fn new() -> Self {
        HtmlTemplate {
            html_body: "".to_string(),
            css_files: Vec::new(),
            js_files: Vec::new(),
            frontend_messages: Vec::new(),
        }
    }


    /// Adding a string to the HTML body
    pub fn push_body(&mut self, body: &str) {
        self.html_body += body;
    }

    /// Add a success message
    pub fn message_success(&mut self, message: String) {
        let fem = FrontendMessage::Success(message);
        self.frontend_messages.push(fem)
    }

    /// Add a warning message
    pub fn message_warning(&mut self, message: String) {
        let fem = FrontendMessage::Warning(message);
        self.frontend_messages.push(fem)
    }

    /// Add an error message
    pub fn message_error(&mut self, message: String) {
        let fem = FrontendMessage::Error(message);
        self.frontend_messages.push(fem)
    }

    /// request a CSS file to be additionally loaded
    pub fn include_css(&mut self, file_path: & 'static str) {
        self.css_files.push(file_path)
    }

    /// request a javascript file to be additionally loaded
    pub fn include_js(&mut self, file_path: & 'static str) {
        self.js_files.push(file_path)
    }
}

impl IntoResponse for HtmlTemplate {

    fn into_response(self) -> Response {
        let mut html = String::new();

        html += "<!DOCTYPE html>\n";
        html += "<html>\n";
        html += "  <head>\n";
        html += "    <meta charset=\"UTF-8\">\n";
        html += "    <meta name=\"color-scheme\" content=\"dark light\">\n";
        html += "    <title>SSLO League</title>\n";
        html += "    <link rel=\"icon\" href=\"/rsc/img/favicon.svg\" sizes=\"any\" type=\"image/svg+xml\">\n";
        html += "    <link rel=\"stylesheet\" href=\"/rsc/css/main.css\">\n";
        for css_file in &self.css_files {
            html += "    <link rel=\"stylesheet\" href=\"";
            html += css_file;
            html += "\">\n";
        }
        html += "    <script src=\"/rsc/js/main.js\" async></script>\n";
        for js_file in &self.js_files {
            html += "    <script src=\"";
            html += js_file;
            html += "\"></script>\n";
        }
        html += "  </head>\n";

        // html body
        html += "  <body><div>\n";

        // page header
        html += "    <header>";
        html += "</header>\n";

        // navigation
        html += "    <nav>\n";
        html += "      <div id=\"NavbarLogo\"><a href=\"/\"><img src=\"/rsc/img/sslo_logo.svg\" title=\"Simracing Sports League Organization\"></a></div>\n";
        html += "      <div id=\"NavbarMenu\">\n";
        html += "          <div class=\"NavbarNoDrop\">\n";
        html += "              <a href=\"/\" class=\"active\">Home</a>\n";
        html += "          </div>\n";
        html += "          <div class=\"NavbarDropdown\">\n";
        html += "              <a href=\"#\" onclick=\"navbarDropdown(this)\">User ⯆</a>\n";
        html += "              <div>\n";
        html += "                  <a href=\"#\">User Settings</a>\n";
        html += "              </div>\n";
        html += "          </div>\n";
        html += "          <div class=\"NavbarLogin\">\n";
        html += "              <a href=\"/html/login\">Login</a>\n";
        html += "          </div>\n";
        html += "      </div>\n";
        html += "    </nav>\n";

        // TODO: implement breadcrumps

        // messages
        html += "<messages>";
        for msg in self.frontend_messages {
            html += &msg.to_html();
        }
        html += "</messages>";

        // content
        html += "    <main>\n";
        html += &self.html_body;
        html += "    </main>\n";

        // footer
        html.push_str("    <footer>\n");
        html.push_str("__FOOTER__");
        html.push_str("    </footer>\n");

        // html finish
        html.push_str("  </div></body>\n");
        html.push_str("</html>\n");

        Html(html).into_response()
    }
}


pub fn create_router(app_state: AppState) -> Router {
    let router = Router::new()
        .route("/rsc/*filepath", routing::get(static_resources::route_handler))

        .route("/", routing::get(routes_html::home::handler))
        .route("/html/login", routing::get(routes_html::login::handler))
        .route("/html/login_email_generate", routing::post(routes_html::login::handler_email_generate))
        .route("/html/login_email_verify/:email/:token", routing::get(routes_html::login::handler_email_verify))
        .route("/html/login_test", routing::get(routes_html::login::handler_login_test))

        .route("/api/v0/login/email", routing::post(routes_rest_v0::login_email::handler))
        .with_state(app_state);
    router
}


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


enum FrontendMessage {
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