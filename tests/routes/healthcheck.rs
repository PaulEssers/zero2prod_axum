use axum::http::StatusCode;
use axum_test_helper::TestClient;
use zero2prod::app::spawn_app;

#[tokio::test]
pub async fn health_check_works() {
    let app = spawn_app().await.expect("Failed to spawn app.");
    let client = TestClient::new(app);
    let res = client.get("/health_check").send().await;
    assert_eq!(res.status(), StatusCode::OK);
}
