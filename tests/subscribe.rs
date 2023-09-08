use axum::http::StatusCode;
use axum_test_helper::TestClient;
use zero2prod::app::spawn_app;
use zero2prod::configuration::get_configuration;
use zero2prod::routes::subscribe;

use sqlx::PgPool;

#[tokio::test]
pub async fn subscribe_returns_200_for_valid_form_data() {
    let configuration = get_configuration().expect("Failed to read configuration.");
    let app = spawn_app(configuration.clone())
        .await
        .expect("Failed to spawn app.");
    let client = TestClient::new(app);

    // Should get the state from app now. How? Or skip this step completely
    // And implement another route to check if the db insertion worked.
    let connection_string = configuration.database.connection_string();
    let connection = PgPool::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres");

    // Start actual test.
    let body = subscribe::NewSubscriber {
        email: String::from("ursula_le_guin@gmail.com"),
        name: String::from("Ursula le Quin"),
    };

    let response = client
        .post("/subscribe")
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await;
    assert_eq!(response.status(), StatusCode::OK);

    let response_db =
        sqlx::query!("SELECT * FROM subscriptions WHERE email='ursula_le_guin@gmail.com'")
            .fetch_all(&connection)
            .await
            .expect("Failed to connect to Postgres when verifying insertion.");

    assert_eq!(response_db.len(), 1)
}

#[tokio::test]
pub async fn subscribe_returns_400_when_data_is_missing() {
    let configuration = get_configuration().expect("Failed to read configuration.");
    let app = spawn_app(configuration)
        .await
        .expect("Failed to spawn app.");
    let client = TestClient::new(app);

    let test_case1 = subscribe::NewSubscriber {
        email: String::from(""),
        name: String::from("Ursula le Quin"),
    };
    let test_case2 = subscribe::NewSubscriber {
        email: String::from("ursula_le_guin@gmail.com"),
        name: String::from(""),
    };
    let test_case3 = subscribe::NewSubscriber {
        email: String::from(""),
        name: String::from(""),
    };

    let test_cases: Vec<(subscribe::NewSubscriber, &str)> = vec![
        (test_case1, "missing the email"),
        (test_case2, "missing the name"),
        (test_case3, "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post("/subscribe")
            .header("Content-Type", "application/json")
            .json(&invalid_body)
            .send()
            .await;
        // .expect("Failed to execute request.");
        // Assert
        assert_eq!(
            StatusCode::BAD_REQUEST,
            response.status(),
            // Additional customised error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}
