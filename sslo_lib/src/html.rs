
struct HtmlTemplate {
    html_body: String,
    css_files: Vec<& 'static str>,
    js_files: Vec<& 'static str>,
}

impl HtmlTemplate {

    /// Create a new, empty HTML template
    pub fn new() -> Self {
        HtmlTemplate {
            html_body: "".to_string(),
            css_files: Vec::new(),
            js_files: Vec::new(),
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
