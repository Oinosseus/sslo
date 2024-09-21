use std::net::{Ipv4Addr, SocketAddr};
use axum::extract::Host;
use axum::handler::HandlerWithoutStateExt;
use axum::http::{StatusCode, Uri};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::{routing, Router};
use crate::{CONFIG_PORT_HTTP, CONFIG_PORT_HTTPS};
use sslo_lib::http_routes::static_resources;

pub mod route_html_login;

struct HtmlTemplate {
    html_body: String,
    css_files: Vec<& 'static str>,
    js_files: Vec<& 'static str>,
}

impl HtmlTemplate {

    pub fn new() -> Self {
        HtmlTemplate {
            html_body: "".to_string(),
            css_files: Vec::new(),
            js_files: Vec::neww(),
        }
    }


    /// Adding a string to the HTML body
    pub fn push_body(&mut self, body: &str) {
        self.html_body += body;
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
        html += "    <link rel=\"icon\" href=\"rsc/img/favicon.svg\" sizes=\"any\" type=\"image/svg+xml\">\n";
        html += "    <link rel=\"stylesheet\" href=\"/rsc/css/main.css\">\n";
        html += "    <script src=\"/rsc/js/main.js\" async></script>\n";
        for css_file in &self.css_files {
            html += "    <link rel=\"stylesheet\" href=\"";
            html += css_file;
            html += "\">\n";
        }
        for js_file in &self.js_files {
            html += "    <script type=\"module\" src=\"";
            html += js_file;
            html += "\" defer></script>\n";
        }
        // for js_file in active_page.javascript_files() {
        //     html += "    <script type=\"module\" src=\"";
        //     html += js_file;
        //     html += "\" defer></script>\n";
        // }
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
        html += "              <a href=\"#\" class=\"active\">Home</a>\n";
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

        // TODO: implement messages
        html += "<messages></messages>";

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



async fn route_main() -> Result<impl IntoResponse, StatusCode> {
    let mut template = HtmlTemplate::new();
    template.push_body("Hello World!");
    Ok(template)
}



pub fn create_router() -> Router {
    let router = Router::new()
        .route("/", routing::get(route_main))
        .route("/html/login", routing::get(route_html_login::handler))
        .route("/rsc/*filepath", routing::get(static_resources::route_handler));
    router
}


#[allow(dead_code)]
pub async fn http2https_background_service() {
    // Implementation from:
    // https://github.com/tokio-rs/axum/blob/main/examples/tls-rustls/src/main.rs

    fn make_https(host: String, uri: Uri) -> Result<Uri, axum::BoxError> {
        let mut parts = uri.into_parts();

        parts.scheme = Some(axum::http::uri::Scheme::HTTPS);

        if parts.path_and_query.is_none() {
            parts.path_and_query = Some("/".parse().unwrap());
        }

        let https_host = host.replace(&CONFIG_PORT_HTTP.to_string(), &CONFIG_PORT_HTTPS.to_string());
        parts.authority = Some(https_host.parse()?);

        Ok(Uri::from_parts(parts)?)
    }

    let redirect = move |Host(host): Host, uri: Uri| async move {
        match make_https(host, uri) {
            Ok(uri) => Ok(Redirect::permanent(&uri.to_string())),
            Err(_) => {
                Err(StatusCode::BAD_REQUEST)
            }
        }
    };

    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, CONFIG_PORT_HTTP));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, redirect.into_make_service())
        .await
        .unwrap();
}