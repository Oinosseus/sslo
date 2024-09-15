use axum::extract::Path;
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/../rsc"]
struct Resources;


pub async fn route_handler(Path(filepath): Path<String>) -> Result<impl IntoResponse, StatusCode> {
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
