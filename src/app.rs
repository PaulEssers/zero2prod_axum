use axum::routing::{get, post};
use axum::Router;
use axum_macros::FromRef;

use crate::routes::subscribe::subscribe;
use crate::routes::utils::health_check;

use sqlx::PgPool;

use std::sync::Arc;

use tower_http::trace::{self, TraceLayer};
use tracing::Level;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub pg_pool: PgPool,
}

pub async fn spawn_app(connection_string: &str) -> Result<Router, String> {
    let connection_pool = PgPool::connect(connection_string)
        .await
        .expect("Failed to connect to Postgres.");

    // Axum starts a service per thread on the machine.
    // Arc lets the database connection be shared between threads
    let shared_state = Arc::new(AppState {
        pg_pool: connection_pool,
    });

    // build our application with some routes
    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscribe", post(subscribe))
        .layer(TraceLayer::new_for_http())
        .with_state(shared_state);

    Ok(app)
}
