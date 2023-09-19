use axum::routing::{get, post};
use axum::Router;
use axum_macros::FromRef;

use crate::configuration::Settings;
use crate::email_client::{EmailClient, ValidEmail};
use crate::routes::confirm::confirm_subscription;
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
    pub base_url: ApplicationBaseUrl,
}

#[derive(Clone, FromRef)]
pub struct ApplicationBaseUrl(pub String);

pub async fn spawn_app(configuration: Settings) -> Result<Router, String> {
    tracing::info!("Creating Postgres connection pool.");
    let pg_pool = PgPool::connect_lazy_with(configuration.database.with_db());

    let sender_email = ValidEmail::new(&configuration.email_client.sender)?;
    let timeout = configuration.email_client.timeout();
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
        timeout,
    );

    let base_url = ApplicationBaseUrl(configuration.application.host);

    // Axum starts a service per thread on the machine.
    // Arc lets the database connection be shared between threads
    let shared_state = Arc::new(AppState {
        pg_pool,
        email_client,
        base_url,
    });

    // build our application with some routes
    tracing::info!("Spawning app.");
    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscribe", post(subscribe))
        .route("/confirm", post(confirm_subscription))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .with_state(shared_state);

    Ok(app)
}
