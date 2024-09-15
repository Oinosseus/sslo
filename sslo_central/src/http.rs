mod route_rsc;

use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::Router;

struct HtmlTemplate {
    html_body: String,
}

impl HtmlTemplate {

    pub fn new() -> Self {
        HtmlTemplate {
            html_body: "".to_string(),
        }
    }


    /// Adding a string to the HTML body
    pub fn push_body(&mut self, body: &str) {
        self.html_body += body;
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
        html += "    <title>SSLO Central</title>\n";
        html += "    <link rel=\"icon\" href=\"rsc/img/favicon.svg\" sizes=\"any\" type=\"image/svg+xml\">\n";
        html += "    <link rel=\"stylesheet\" href=\"/rsc/css/main.css\">\n";
        html += "    <script src=\"/rsc/js/main.js\" async></script>\n";
        // for css_file in &self.css_files {
        //     html += "    <link rel=\"stylesheet\" href=\"";
        //     html += css_file;
        //     html += "\">\n";
        // }
        // for js_file in &self.js_files {
        //     html += "    <script type=\"module\" src=\"";
        //     html += js_file;
        //     html += "\" defer></script>\n";
        // }
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
        html += "      <div id=\"NavbarLogo\"><img src=\"/rsc/img/sslo_logo.svg\"></div>\n";
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
        html += "              <a href=\"#\">Login</a>\n";
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
        .route("/", axum::routing::get(route_main))
        .route("/rsc/*filepath", axum::routing::get(route_rsc::route_handler_rsc));
    router
}
