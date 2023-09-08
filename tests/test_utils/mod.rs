//use axum_test_helper::TestClient;
//use zero2prod::app::spawn_app;
//use zero2prod::configuration::get_configuration;

// Figure out how to pass the connection pool this way, or move back to all individual functions...
//pub async fn create_test_client() -> TestClient {
//    let configuration = get_configuration().expect("Failed to read configuration.");
//    let app = spawn_app(configuration)
//        .await
//        .expect("Failed to spawn app.");
//
//}
