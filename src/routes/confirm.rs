use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum_macros::debug_handler;
use uuid::Uuid;



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
    StatusCode::OK
}

