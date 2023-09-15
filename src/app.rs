use axum::routing::{get, post};
use axum::Router;
use axum_macros::FromRef;

use crate::configuration::Settings;
use crate::email_client::{EmailClient, ValidEmail};
use crate::routes::subscribe::subscribe;
use crate::routes::utils::health_check;

use sqlx::PgPool;
use std::sync::Arc;
use tower_http::trace;
use tower_http::trace::TraceLayer;
use tracing::Level;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub pg_pool: PgPool,
    pub email_client: EmailClient,
}

pub async fn spawn_app(configuration: Settings) -> Result<Router, String> {
    tracing::info!("Creating Postgres connection pool.");
    let pg_pool = PgPool::connect_lazy_with(configuration.database.with_db());

    let sender_email = ValidEmail::new(&configuration.email_client.sender)?;
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
    );

    // Axum starts a service per thread on the machine.
    // Arc lets the database connection be shared between threads
    let shared_state = Arc::new(AppState {
        pg_pool,
        email_client,
    });

    // build our application with some routes
    tracing::info!("Spawning app.");
    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscribe", post(subscribe))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .with_state(shared_state);

    Ok(app)
}
