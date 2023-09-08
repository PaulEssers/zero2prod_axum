use axum_test_helper::TestClient;
use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgPool};
use uuid::Uuid;
use zero2prod::app::spawn_app;
use zero2prod::configuration::get_configuration;
use zero2prod::configuration::DatabaseSettings;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

// Ensure that the `tracing` stack is only initialised once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    let subscriber = get_subscriber("test".into(), "debug".into());
    init_subscriber(subscriber);
});

pub struct TestSetup {
    pub client: TestClient,
    pub pg_pool: PgPool,
}

pub async fn create_test_setup() -> TestSetup {
    // The first time `initialize` is invoked the code in `TRACING` is executed.
    // All other invocations will instead skip execution.
    Lazy::force(&TRACING);

    let mut configuration = get_configuration().expect("Failed to read configuration.");

    // Generate a fresh database for each test.
    let new_db = format!("testdb-{:?}", Uuid::new_v4());
    println!("Creating new database for current test: {:?}", new_db);
    configuration.database.database_name = new_db;
    configure_database(&configuration.database).await;

    // Spawn the app with the newly created db.
    // Can I get the pool back from the app? Now I'm creating multiple pools.
    let connection_string = configuration.database.connection_string();
    let app = spawn_app(&connection_string)
        .await
        .expect("Failed to spawn app.");
    let client = TestClient::new(app);

    let pg_pool = PgPool::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres");

    TestSetup { client, pg_pool }
}

// Creates and migrates a new database to be used for testing.
pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let connection = PgPool::connect(&config.connection_string_no_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");
    // Migrate database (run commands defined in ./migrations)
    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");
    connection_pool
}
