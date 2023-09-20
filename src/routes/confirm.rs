use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum_macros::debug_handler;
use uuid::Uuid;
use sqlx::PgPool;


use std::sync::Arc;

use crate::app;
use crate::models;





#[debug_handler]
#[tracing::instrument(
    name = "Confirming Subscription",
    skip(state, query),
    fields(
        // could not find a way to get this automatically with tower_http like in the book.
        request_id = %Uuid::new_v4(), 
        token = %query.get_token(),
        // subscriber_name= %payload.get_name()
    )
)]
pub async fn confirm_subscription(
    State(state): State<Arc<app::AppState>>,
    Query(query): Query<models::TokenQuery>,
) -> StatusCode {
    tracing::info!("Processing request: {:?}", query);
    
    let id = match get_subscriber_id_from_token(&state.pg_pool, query.get_token()).await {
        Ok(id) => id,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };
    match id {
        // Non-existing token!
        None => StatusCode::UNAUTHORIZED,
        Some(subscriber_id) => {
            if confirm_subscriber(&state.pg_pool, subscriber_id).await.is_err() {
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
            StatusCode::OK
        }
    }



}

#[tracing::instrument(
    name = "Mark subscriber as confirmed",
    skip(subscriber_id, pool)
)]
pub async fn confirm_subscriber(
    pool: &PgPool,
    subscriber_id: Uuid
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id,
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}

#[tracing::instrument(
    name = "Get subscriber_id from token",
    skip(subscription_token, pool)
)]
pub async fn get_subscriber_id_from_token(
    pool: &PgPool,
    subscription_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT subscriber_id FROM subscription_tokens WHERE subscription_token = $1"#,
        subscription_token,
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
        })?;
    Ok(result.map(|r| r.subscriber_id))
}