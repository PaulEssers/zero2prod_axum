use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};

use chrono::Utc;
use uuid::Uuid;

use std::sync::Arc;

use crate::app;

#[derive(Serialize, Deserialize, Debug)]
pub struct NewSubscriber {
    pub email: String,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct Test {
    result: i32,
}

#[debug_handler]
pub async fn subscribe(
    State(state): State<Arc<app::AppState>>,
    Json(payload): Json<NewSubscriber>,
) -> StatusCode {
    tracing::info!("Processing request: {:?}", payload);

    let res = sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at)
            VALUES ($1, $2, $3, $4)
            "#,
        Uuid::new_v4(),
        payload.email,
        payload.name,
        Utc::now()
    )
    .execute(&state.pg_pool)
    .await;

    match res {
        Ok(_) => StatusCode::OK,
        Err(err) => {
            println!("Insertion failed with error: {:?}", err);
            StatusCode::BAD_REQUEST
        }
    }
}
