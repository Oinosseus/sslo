use axum::extract::FromRequestParts;
use axum::http::header;
use axum::http::request::Parts;

struct WebsiteUser {

}

#[axum::async_trait]
impl<S> FromRequestParts<S> for WebsiteUser
where
    S: Send + Sync,
{
    type Rejection = ();

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let cookie: String = if let parts.headers.get(header::COOKIE) {

        } else {

        };

        println!("HERE {}", cookie);
        todo!();
    }
}