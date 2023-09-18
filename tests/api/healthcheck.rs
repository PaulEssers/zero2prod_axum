use crate::test_utils;
use axum::http::StatusCode;

#[tokio::test]
pub async fn health_check_works() {
    let test_setup = test_utils::create_test_setup().await;

    let res = test_setup.client.get("/health_check").send().await;
    assert_eq!(res.status(), StatusCode::OK);
}
