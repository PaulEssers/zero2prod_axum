use axum::http::StatusCode;
use axum_test_helper::TestClient;
use zero2prod::app::spawn_app;

use sqlx::{Connection, PgConnection};
use zero2prod::configuration::get_configuration;

#[tokio::test]
pub async fn subscribe_returns_200_for_valid_form_data() {
    let app = spawn_app().await.expect("Failed to spawn app.");
    let client = TestClient::new(app);

    let configuration = get_configuration.expect("Failed to read configuration.");
    let connection_string = configuration.database.connection_string();

    let connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres");

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let response = client
        .post("/subscribe")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send()
        .await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
pub async fn subscribe_returns_400_when_data_is_missing() {
    let app = spawn_app().await.expect("Failed to spawn app.");
    let client = TestClient::new(app);

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post("/subscribe")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");
        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            // Additional customised error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}
