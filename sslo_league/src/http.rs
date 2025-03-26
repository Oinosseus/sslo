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
mod http_user;

struct HtmlTemplate {
    html_body: String,
    css_files: Vec<& 'static str>,
    js_files: Vec<& 'static str>,
    frontend_messages: Vec<FrontendMessage>,
    http_user: http_user::HttpUser,
}

impl HtmlTemplate {

    pub fn new(http_user: http_user::HttpUser) -> Self {
        HtmlTemplate {
            html_body: "".to_string(),
            css_files: Vec::new(),
            js_files: Vec::new(),
            frontend_messages: Vec::new(),
            http_user,
        }
    }


    pub fn http_user(&self) -> &http_user::HttpUser {
        &self.http_user
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

    pub async fn into_response(self) -> Response {
        let mut html = String::new();

        html += "<!DOCTYPE html>";
        html += "<html>";
        html += "  <head>";
        html += "    <meta charset=\"UTF-8\">";
        html += "    <meta name=\"color-scheme\" content=\"dark light\">";
        html += "    <title>SSLO League</title>";
        html += "    <link rel=\"icon\" href=\"/rsc/img/favicon.svg\" sizes=\"any\" type=\"image/svg+xml\">";
        html += "    <link rel=\"stylesheet\" href=\"/rsc/css/main.css\">";
        for css_file in &self.css_files {
            html += "    <link rel=\"stylesheet\" href=\"";
            html += css_file;
            html += "\">";
        }
        html += "    <script src=\"/rsc/js/main.js\" defer></script>";
        for js_file in &self.js_files {
            html += "    <script src=\"";
            html += js_file;
            html += "\" defer></script>";
        }
        html += "  </head>";
        html += "  <body>";

        // busy spinner
        html += "<div id=\"BusySpinner\">";
        html += "<div><svg viewBox=\"-1 -2 88 36\">";
        html += "<g transform=\"translate(-2.4746312,-74.753705)\">";
        html += "<path id=\"BusySpinnerRedTop\" style=\"fill:#e52115;fill-opacity:1;stroke:none;stroke-width:0.264583px;stroke-linecap:butt;stroke-linejoin:miter;stroke-opacity:1\" d=\"M 4.6931537,91.168273 2.4746312,83.253777 84.608969,78.156731 80.454171,88.769797 Z\" />";
        html += "<path id=\"BusySpinnerRedBottom\" style=\"fill:#e52115;fill-opacity:1;stroke:none;stroke-width:0.264583px;stroke-linecap:butt;stroke-linejoin:miter;stroke-opacity:1\" d=\"M 2.916956,100.28772 4.9668335,92.353875 80.861309,90.116 l 6.163959,14.28806 z\" />";
        html += "<path id=\"BusySpinnerS1Bg\" style=\"display:inline;fill:#004600;fill-opacity:1;stroke:none;stroke-width:0.264583px;stroke-linecap:butt;stroke-linejoin:miter;stroke-opacity:1\" d=\"M 10.662993,79.128101 9.9900573,93.071494 11.3133,93.868482 8.8740882,106.23914 31.770429,104.42775 30.214801,86.663479 31.169867,85.907215 27.869605,75.943469 Z\"/>";
        html += "<path id=\"BusySpinnerS2Bg\" style=\"fill:#004600;fill-opacity:1;stroke:none;stroke-width:0.264583px;stroke-linecap:butt;stroke-linejoin:miter;stroke-opacity:1\" d=\"m 28.396062,105.93963 20.134337,-2.49347 -1.461294,-18.907622 0.736878,-0.487128 -1.121232,-9.249898 -4.032255,0.623115 -11.668283,0.368656 -0.233722,17.08661 -0.980302,0.388673 z\" />";
        html += "<path id=\"BusySpinnerS2Fg\" style=\"fill:#ffffff;fill-opacity:1;stroke:none;stroke-width:0.264583px;stroke-linecap:butt;stroke-linejoin:miter;stroke-opacity:1\" d=\"m 29.144261,105.05352 1.053166,-11.472878 9.810955,1.23217 0.977738,-0.917921 -9.820474,-0.509207 0.224637,-17.241837 14.979443,-0.752482 1.046523,8.64909 -8.54792,0.727555 -1.07929,0.714618 9.049445,-1.176597 1.349771,18.711809 z\" />";
        html += "<path id=\"BusySpinnerOBg\" style=\"fill:#004600;fill-opacity:1;stroke:none;stroke-width:0.264583px;stroke-linecap:butt;stroke-linejoin:miter;stroke-opacity:1\" d=\"m 57.897911,74.902317 c 0,0 -1.255337,12.801036 -1.230957,31.487323 0,0 18.267263,0.34155 18.339329,-0.001 C 76.966937,97.055797 74.493847,78.573428 73.897433,75.086425 58.488165,74.467013 57.897911,74.902317 57.897911,74.902317 Z\" />";
        html += "<path id=\"BusySpinnerOFg\" style=\"display:inline;fill:#ffffff;fill-opacity:1;stroke:none;stroke-width:0.264583px;stroke-linecap:butt;stroke-linejoin:miter;stroke-opacity:1\" d=\"m 58.63426,75.516093 c -2.276455,12.707054 -1.588301,30.675647 -1.588301,30.675647 0,0 13.762354,0.36183 17.925367,-0.53684 1.204978,-23.315684 -1.729176,-30.233937 -1.729176,-30.233937 0,0 -6.021829,-0.797607 -14.60789,0.09513 z\" />";
        html += "<path id=\"BusySpinnerLBg\" style=\"display:inline;fill:#004600;fill-opacity:1;stroke:none;stroke-width:0.264583px;stroke-linecap:butt;stroke-linejoin:miter;stroke-opacity:1\" d=\"m 54.005209,75.467091 0.202957,10.734361 4.481291,-0.07349 0.76304,18.079168 -0.518408,0.64761 c 0,0 -9.562832,0.63117 -9.754594,0.63117 -0.191759,0 -2.975382,-0.61349 -2.975382,-0.61349 l 2.317589,-12.658526 -1.507366,-17.360473 3.51512,0.466249 z\" />";
        html += "<path id=\"BusySpinnerS1Fg\" style=\"fill:#ffffff;fill-opacity:1;stroke:#004600;stroke-width:0.565;stroke-linecap:butt;stroke-linejoin:miter;stroke-dasharray:none;stroke-opacity:1;paint-order:stroke fill markers\" d=\"M 10.948314,79.544703 10.511495,92.948831 24.412259,93.894348 23.991024,94.826544 11.65984,93.626175 9.5858333,105.75077 31.467195,103.82706 29.746565,87.849773 29.495232,85.859715 17.887637,87.579842 v -1.137285 l 12.666714,-0.77084 -2.925776,-8.949491 z\" />";
        html += "<path id=\"BusySpinnerLFg\" style=\"fill:#ffffff;fill-opacity:1;stroke:none;stroke-width:0.264583px;stroke-linecap:butt;stroke-linejoin:miter;stroke-opacity:1\" d=\"m 47.451705,75.270888 1.413316,17.714086 -2.379222,12.086846 12.648936,-0.59286 -0.872692,-18.26329 -4.263853,0.321316 0.004,1.747123 -0.304239,0.327299 0.128702,-12.73805 z\" />";
        html += "<path id=\"BusySpinnerORing\" style=\"fill:#004600;fill-opacity:1;stroke:none;stroke-width:0.264583px;stroke-linecap:butt;stroke-linejoin:miter;stroke-opacity:1\" d=\"m 63.699513,85.579766 4.652704,0.742514 -1.345161,9.792359 -1.113414,-9.18118 z\" />";
        html += "</g>";
        html += "</svg></div>";
        html += "</div>";

        // html body
        html += "  <div id=\"BodyDiv\">";

        // page header
        html += "    <header>";
        html += "</header>";

        // navigation
        html += "    <nav>";
        html += "      <div id=\"NavbarLogo\"><a href=\"/\"><img src=\"/rsc/img/sslo_logo.svg\" title=\"Simracing Sports League Organization\"></a></div>";
        html += "      <div id=\"NavbarMenu\">";
        html += "          <div class=\"NavbarNoDrop\">";
        html += "              <a href=\"/\" class=\"active\">Home</a>";
        html += "          </div>";
        html += "          <div class=\"NavbarDropdown\">";
        html += "              <a href=\"#\" onclick=\"navbarDropdown(this)\">League ⯆</a>";
        html += "              <div>";
        html += "                  <a href=\"/html/ranking\">Driver Ranking</a>";
        html += "                  <a href=\"/html/schedules\">Scheduled Races</a>";
        html += "                  <a href=\"/html/championships\">Championships</a>";
        html += "              </div>";
        html += "          </div>";
        html += "          <div class=\"NavbarDropdown\">";
        html += "              <a href=\"#\" onclick=\"navbarDropdown(this)\">Content ⯆</a>";
        html += "              <div>";
        html += "                  <a href=\"/html/users\">Users</a>";
        html += "                  <a href=\"/html/tracks\">Tracks</a>";
        html += "                  <a href=\"/html/cars\">Cars</a>";
        html += "                  <a href=\"/html/cars\">Car Classes</a>";
        html += "              </div>";
        html += "          </div>";
        if self.http_user.is_logged_in() {
            html += "          <div class=\"NavbarDropdown\">";
            html += "              <a href=\"#\" onclick=\"navbarDropdown(this)\">User ⯆</a>";
            html += "              <div>";
            html += "                  <a href=\"/html/user_profile\">Profile</a>";
            html += "                  <a href=\"/html/user/accounts\">Accounts</a>";
            html += "                  <a href=\"/html/logout\">Logout</a>";
            html += "              </div>";
            html += "          </div>";
        } else {
            html += "          <div class=\"NavbarLogin\">";
            html += "              <a href=\"/html/login\">Login</a>";
            html += "          </div>";
        }
        html += "          <div class=\"NavbarDropdown\">";
        html += "              <a href=\"#\" onclick=\"navbarDropdown(this)\">About ⯆</a>";
        html += "              <div>";
        html += "                  <a href=\"/html/about\">General</a>";
        html += "                  <a href=\"/html/about/third_party\">Third Party Integrations</a>";
        html += "                  <a href=\"/html/about/data_protection\">Data Protection</a>";
        html += "              </div>";
        html += "          </div>";
        html += "      </div>";
        html += "    </nav>";

        // TODO: implement breadcrumps

        // messages
        html += "<messages>";
        for msg in self.frontend_messages {
            html += &msg.to_html();
        }
        html += "</messages>";

        // content
        html += "    <main>";
        html += &self.html_body;
        html += "    </main>";

        // footer
        html.push_str("    <footer>");
        html.push_str(&self.http_user.user.html_name().await);
        html.push_str(" <small>&lt;");
        html.push_str(self.http_user.user.activity().await.label());
        let promotion = self.http_user.user.promotion().await.label();
        if promotion.len() > 0 { html.push_str(" "); }
        html.push_str(promotion);
        html.push_str("&gt;</small>");
        html.push_str("    </footer>");

        // html finish
        html.push_str("  </div></body>");
        html.push_str("</html>");

        Html(html).into_response()
    }
}


pub fn create_router(app_state: AppState) -> Router {
    let router = Router::new()
        .route("/rsc/*filepath", routing::get(static_resources::route_handler))

        .route("/", routing::get(routes_html::home::handler))

        .route("/html/login", routing::get(routes_html::login::handler))
        .route("/html/login_email_create/:email", routing::get(routes_html::login::handler_email_create))
        .route("/html/login_email_existing/:email", routing::get(routes_html::login::handler_email_existing))
        .route("/html/login_email_verify/:email/:token", routing::get(routes_html::login::handler_email_verify))
        .route("/html/login_steam_create", routing::get(routes_html::login::handler_steam_create))
        .route("/html/login_steam_existing", routing::get(routes_html::login::handler_steam_existing))
        .route("/html/login_steam_assign", routing::get(routes_html::login::handler_steam_assign))
        .route("/html/logout", routing::get(routes_html::login::handler_logout))

        .route("/html/user_profile", routing::get(routes_html::user::handler_profile))
        .route("/html/user/accounts", routing::get(routes_html::user::accounts::handler))

        .route("/api/v0/login/password", routing::post(routes_rest_v0::login_password::handler))
        .route("/api/v0/user/set_password", routing::post(routes_rest_v0::user::handler_set_password))
        .route("/api/v0/user/set_name", routing::post(routes_rest_v0::user::handler_set_name))
        .route("/api/v0/user/account/email", routing::put(routes_rest_v0::user::account::email_put))
        .route("/api/v0/user/account/email", routing::delete(routes_rest_v0::user::account::email_delete))

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