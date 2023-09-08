use axum::routing::{get, post};
use axum::Router;
use axum_macros::FromRef;

use crate::routes::subscribe::subscribe;
use crate::routes::utils::health_check;

use sqlx::PgPool;

use std::sync::Arc;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub pg_pool: PgPool,
}

pub async fn spawn_app(connection_string: &str) -> Result<Router, String> {
    let connection_pool = PgPool::connect(connection_string)
        .await
        .expect("Failed to connect to Postgres.");

    let shared_state = Arc::new(AppState {
        pg_pool: connection_pool,
    });

    // build our application with some routes
    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscribe", post(subscribe))
        .with_state(shared_state);

    Ok(app)
}
