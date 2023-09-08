use axum::Error;
use std;
use std::net::SocketAddr;

use zero2prod::app::spawn_app;
use zero2prod::configuration::get_configuration;

mod telemetry;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let subscriber = telemetry::get_subscriber("zero2prod".into(), "info".into());
    telemetry::init_subscriber(subscriber);

    // read the configuration file
    let configuration = get_configuration().expect("Failed to read configuration.");

    // spawn the app.
    let app = spawn_app(&configuration.database.connection_string())
        .await
        .expect("Failed to initialize app.");

    // Serve it with hyper.
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Failed to start server.");

    Ok(())
}
