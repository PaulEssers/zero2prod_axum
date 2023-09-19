use axum_test_helper::TestClient;
use once_cell::sync::Lazy;
use sqlx::{Executor, PgPool}; // Connection,
use std;
use std::net::IpAddr;
use tracing::info;
use uuid::Uuid;
use wiremock::MockServer;
use zero2prod::app::spawn_app;
use zero2prod::configuration::get_configuration;
use zero2prod::configuration::DatabaseSettings;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

// use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
// use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

// Ensure that the `tracing` stack is only initialised once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    // We cannot assign the output of `get_subscriber` to a variable based on the value of `TEST_LOG`

    // If TEST_LOG is set, output the logs to stdout, else do not show them.
    // because the sink is part of the type returned by `get_subscriber`, therefore they are not the
    // same type. We could work around it, but this is the most straight-forward way of moving forward.
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_sink_subscriber(subscriber_name, default_filter_level);
        init_subscriber(subscriber);
    };
});

// Could not get this to work as a parameter in the function above, so just copied it
// this version sends the traces to std::io::sink, for use in tests.
pub fn get_sink_subscriber(name: String, env_filter: String) -> impl Subscriber + Send + Sync {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let formatting_layer = BunyanFormattingLayer::new(name, std::io::sink);
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

pub struct TestSetup {
    pub client: TestClient,
    pub pg_pool: PgPool,
    pub email_server: MockServer,
}

#[tracing::instrument()]
pub async fn create_test_setup() -> TestSetup {
    // The first time `initialize` is invoked the code in `TRACING` is executed.
    // All other invocations will instead skip execution.
    Lazy::force(&TRACING);

    info!("Reading configuration settings.");
    let mut configuration = get_configuration().expect("Failed to read configuration.");

    // Generate a fresh database for each test.
    let new_db = format!("testdb-{:?}", Uuid::new_v4());
    info!("Creating new database for current test: {:?}", new_db);
    configuration.database.database_name = new_db;
    configure_database(&configuration.database).await;

    // Launch a mock server to stand in for Postmark's service
    let email_server = MockServer::start().await;
    // Extract the randomly chosen port and pass it to the email_client
    let port = email_server.address().port();
    let ip = match email_server.address().ip() {
        IpAddr::V4(ipv4) => ipv4.to_string(),
        IpAddr::V6(ipv6) => ipv6.to_string(),
    };
    // !!! IP address still contains quotes, due to my format call?
    info!(ip);
    info!(
        "Creating mock email server on address: {}:{:?}",
        remove_quotes(&ip),
        port
    );

    configuration.email_client.base_url = format!("http://{}:{:?}", remove_quotes(&ip), port);

    // Spawn the app with the newly created db.
    // Can I get the pool back from the app? Now I'm creating multiple pools.
    info!("Spawning app.");
    let app = spawn_app(configuration.clone())
        .await
        .expect("Failed to spawn app.");
    let client = TestClient::new(app);

    // This pool is required to directly check the result of database operations.
    info!("Creating extra postgres connection pool for checking database operations.");
    let connection_options = configuration.database.with_db();
    let pg_pool = PgPool::connect_with(connection_options)
        .await
        .expect("Failed to connect to Postgres");

    TestSetup {
        client,
        pg_pool,
        email_server,
    }
}

// Creates and migrates a new database to be used for testing.
pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let connection = PgPool::connect_with(config.without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");
    // Migrate database (run commands defined in ./migrations)
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");
    connection_pool
}

fn remove_quotes(input: &str) -> String {
    let mut result = String::new();

    for c in input.chars() {
        if c != '"' {
            result.push(c);
        }
    }

    result
}
