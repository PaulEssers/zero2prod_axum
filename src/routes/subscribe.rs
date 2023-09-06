use axum::body::Body;
use axum::extract::Json;
use axum::http::{Response, StatusCode};
use axum_macros::debug_handler;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct NewSubscriber {
    email: String,
    name: String,
}

#[debug_handler]
pub async fn subscribe(Json(payload): Json<NewSubscriber>) -> Response<Body> {
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap()
}
