use axum::http::{StatusCode, Response};
use axum::body::Body;
use axum_macros::debug_handler;

#[debug_handler]
pub async fn health_check() -> Response<Body> {
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap()
}