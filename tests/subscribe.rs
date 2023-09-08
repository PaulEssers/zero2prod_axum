use axum::http::StatusCode;
use axum_test_helper::TestClient;
use serde::Serialize;
use sqlx::PgPool;
use zero2prod::app::spawn_app;
use zero2prod::configuration::get_configuration;
use zero2prod::routes::subscribe;

mod test_utils;

#[tokio::test]
pub async fn subscribe_returns_200_for_valid_form_data() {
    let test_setup = test_utils::create_test_setup().await;

    let body = subscribe::NewSubscriber {
        email: String::from("ursula_le_guin@gmail.com"),
        name: String::from("Ursula le Quin"),
    };

    let response = test_setup
        .client
        .post("/subscribe")
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await;
    assert_eq!(response.status(), StatusCode::OK);

    let response_db =
        sqlx::query!("SELECT * FROM subscriptions WHERE email='ursula_le_guin@gmail.com'")
            .fetch_all(&test_setup.pg_pool)
            .await
            .expect("Failed to connect to Postgres when verifying insertion.");

    assert_eq!(response_db.len(), 1)
}

// Could not get these tests to run in a loop due to the varying structs needed to
// mimic missing keys in the JSON payload.
#[derive(Serialize)]
struct MissingEmail {
    name: String,
}

#[tokio::test]
pub async fn subscribe_returns_422_when_email_is_missing() {
    let test_setup = test_utils::create_test_setup().await;

    let json = MissingEmail {
        name: String::from("Ursula le Quin"),
    };

    let response = test_setup
        .client
        .post("/subscribe")
        .header("Content-Type", "application/json")
        .json(&json)
        .send()
        .await;
    // .expect("Failed to execute request.");
    // Assert
    assert_eq!(
        StatusCode::UNPROCESSABLE_ENTITY,
        response.status(),
        // Additional customised error message on test failure
        "The API did not fail with 422 Unprocessable Entity when the payload was missing the email address."
    );
}

#[derive(Serialize)]
struct MissingName {
    email: String,
}

#[tokio::test]
pub async fn subscribe_returns_422_when_name_is_missing() {
    let test_setup = test_utils::create_test_setup().await;

    let json = MissingName {
        email: String::from("ursula_le_quin@gmail.com"),
    };

    let response = test_setup
        .client
        .post("/subscribe")
        .header("Content-Type", "application/json")
        .json(&json)
        .send()
        .await;
    // .expect("Failed to execute request.");
    // Assert
    assert_eq!(
        StatusCode::UNPROCESSABLE_ENTITY,
        response.status(),
        // Additional customised error message on test failure
        "The API did not fail with 422 Unprocessable Entity when the payload was missing the name."
    );
}

#[derive(Serialize)]
struct MissingBoth {}

#[tokio::test]
pub async fn subscribe_returns_422_when_email_and_name_are_missing() {
    let test_setup = test_utils::create_test_setup().await;

    let json = MissingBoth {};

    let response = test_setup
        .client
        .post("/subscribe")
        .header("Content-Type", "application/json")
        .json(&json)
        .send()
        .await;
    // .expect("Failed to execute request.");
    // Assert
    assert_eq!(
        StatusCode::UNPROCESSABLE_ENTITY,
        response.status(),
        // Additional customised error message on test failure
        "The API did not fail with 422 Unprocessable Entity when the payload was missing both the name and the email address."
    );
}
