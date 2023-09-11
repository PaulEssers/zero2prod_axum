use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum_macros::debug_handler;
use chrono::Utc;
use uuid::Uuid;

use std::sync::Arc;

use crate::app;

use crate::models;

#[debug_handler]
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(state, payload),
    fields(
        // could not find a way to get this automatically with tower_http like in the book.
        request_id = %Uuid::new_v4(), 
        subscriber_email = %payload.get_email(),
        subscriber_name= %payload.get_name()
    )
)]
pub async fn subscribe(
    State(state): State<Arc<app::AppState>>,
    Json(payload): Json<models::NewSubscriber>,
) -> StatusCode {
    tracing::info!("Processing request: {:?}", payload);

    let res = sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at)
            VALUES ($1, $2, $3, $4)
            "#,
        Uuid::new_v4(),
        payload.get_email(),
        payload.get_name(),
        Utc::now()
    )
    .execute(&state.pg_pool)
    .await;

    match res {
        Ok(_) => StatusCode::OK,
        Err(err) => {
            tracing::error!("Insertion failed with error: {:?}", err);
            StatusCode::BAD_REQUEST
        }
    }
}
