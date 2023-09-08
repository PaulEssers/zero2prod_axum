use axum::http::StatusCode;
use axum_test_helper::TestClient;
use test_utils::create_test_setup;
use zero2prod::app::spawn_app;
use zero2prod::configuration::get_configuration;

mod test_utils;

#[tokio::test]
pub async fn health_check_works() {
    let test_setup = test_utils::create_test_setup().await;

    let res = test_setup.client.get("/health_check").send().await;
    assert_eq!(res.status(), StatusCode::OK);
}
