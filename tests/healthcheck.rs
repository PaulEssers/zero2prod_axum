use axum::http::StatusCode;
use axum_test_helper::TestClient;
use zero2prod::app::spawn_app;
use zero2prod::configuration::get_configuration;

mod test_utils;

#[tokio::test]
pub async fn health_check_works() {
    let configuration = get_configuration().expect("Failed to read configuration.");
    let app = spawn_app(configuration)
        .await
        .expect("Failed to spawn app.");
    let client = TestClient::new(app);
    // let client = test_utils::create_test_client().await;

    let res = client.get("/health_check").send().await;
    assert_eq!(res.status(), StatusCode::OK);
}
