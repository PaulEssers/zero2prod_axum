use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("sqlx: {0}")]
    PostgreSQL(#[from] sqlx::error::Error),
    #[error("internal server error")]
    Internal,
    #[error("axum: {0}")]
    Axum(#[from] axum::Error),
    #[error("url: {0}")]
    Url(#[from] url::ParseError),
    #[error("reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        println!("->> {:<12} - {self:?}", "INTO_RES");

        // Create a placeholder Axum response.
        let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();

        // Insert the Error into the response.
        response.extensions_mut().insert(self);

        response
    }
}
