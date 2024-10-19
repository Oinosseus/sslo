use axum::http::StatusCode;
use axum::Json;
use rand::RngCore;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct RequestData {
    email: String,
}

#[derive(Serialize)]
pub struct ResponseData {
}

pub async fn handler(Json(input): Json<RequestData>) -> Result<Json<ResponseData>, StatusCode> {

    // artificial slowdown for 2..5s
    let wait_ms: u64 = 1000u64 + u64::from(rand::thread_rng().next_u32()) / 0x200_000u64; // should result in ~2000 maximum
    println!("wait_ms: {}", wait_ms);
    tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;
    return Ok(ResponseData{}.into());

    // create new token
    todo!();

    // check if email already exist, read last_usage timestamp
    todo!();

    // create email db table row
    todo!();

    // update token
    todo!();

    // send information email
    todo!();

    Err(StatusCode::INTERNAL_SERVER_ERROR)
}
