use crate::test_utils;
use axum::http::StatusCode;
use axum_test_helper::TestResponse;
use serde::Serialize;

use wiremock::matchers::{any, method, path};
use wiremock::{Mock, ResponseTemplate};

// This trait flags a struct as valid input for TestSetup.subscribe_request
pub trait SubscribeRequestBody {}

#[derive(serde::Serialize)]
pub struct SubscribeRequest {
    email: String,
    name: String,
}
impl SubscribeRequestBody for SubscribeRequest {}

impl test_utils::TestSetup {
    pub async fn post_subscriptions<T>(&self, body: &T) -> TestResponse
    where
        T: SubscribeRequestBody + serde::Serialize,
    {
        self.client
            .post("/subscribe")
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
    }
}

#[tokio::test]
// #[quickcheck_macros::quickcheck]
//#[quickcheck_async::tokio]
pub async fn subscribe_returns_200_for_valid_form_data() {
    let test_setup = test_utils::create_test_setup().await;

    let body = SubscribeRequest {
        email: String::from("ursula_le_guin@gmail.com"),
        name: String::from("Ursula le Quin"),
    };

    // email server for the confirmation mail, without it the subscribe route will fail.
    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_setup.email_server)
        .await;

    let response = test_setup.post_subscriptions(&body).await;
    assert_eq!(response.status(), StatusCode::OK);
    // response.status() == StatusCode::OK
}

#[tokio::test]
pub async fn subscribe_inserts_rows_into_database() {
    let test_setup = test_utils::create_test_setup().await;

    let body = SubscribeRequest {
        email: String::from("ursula_le_guin@gmail.com"),
        name: String::from("Ursula le Quin"),
    };

    // email server for the confirmation mail, without it the subscribe route will fail.
    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_setup.email_server)
        .await;

    let response = test_setup.post_subscriptions(&body).await;
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
impl SubscribeRequestBody for MissingEmail {}

#[tokio::test]
pub async fn subscribe_returns_422_when_email_is_missing() {
    let test_setup = test_utils::create_test_setup().await;

    let json = MissingEmail {
        name: String::from("Ursula le Quin"),
    };

    let response = test_setup.post_subscriptions(&json).await;
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
impl SubscribeRequestBody for MissingName {}

#[tokio::test]
pub async fn subscribe_returns_422_when_name_is_missing() {
    let test_setup = test_utils::create_test_setup().await;

    let json = MissingName {
        email: String::from("ursula_le_quin@gmail.com"),
    };

    let response = test_setup.post_subscriptions(&json).await;
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
impl SubscribeRequestBody for MissingBoth {}

#[tokio::test]
pub async fn subscribe_returns_422_when_email_and_name_are_missing() {
    let test_setup = test_utils::create_test_setup().await;

    let json = MissingBoth {};

    let response = test_setup.post_subscriptions(&json).await;
    // .expect("Failed to execute request.");
    // Assert
    assert_eq!(
        StatusCode::UNPROCESSABLE_ENTITY,
        response.status(),
        // Additional customised error message on test failure
        "The API did not fail with 422 Unprocessable Entity when the payload was missing both the name and the email address."
    );
}

#[tokio::test]
async fn subscribe_returns_a_422_when_fields_are_present_but_erroneous() {
    // Arrange
    let test_setup = test_utils::create_test_setup().await;

    let test_cases = vec![
        ("", "ursula_le_guin@gmail.com", "empty name"),
        ("Ursula le Guin", "", "empty email"),
        ("Ursula", "definitely-not-an-email", "invalid email"),
    ];
    for (name, email, description) in test_cases {
        // Act

        let body = SubscribeRequest {
            email: String::from(email),
            name: String::from(name),
        };

        let response = test_setup.post_subscriptions(&body).await;
        assert_eq!(
            response.status(),
            StatusCode::UNPROCESSABLE_ENTITY,
            "The API did not return a 422 Unprocessable Entity when the payload had an {}.",
            description
        );
    }
}

#[tokio::test]
pub async fn subscribe_sends_a_confirmation_mail_for_valid_data() {
    let test_setup = test_utils::create_test_setup().await;

    let body = SubscribeRequest {
        email: String::from("ursula_le_guin@gmail.com"),
        name: String::from("Ursula le Quin"),
    };

    // Set up the mock server and tell it what the request should look like.
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_setup.email_server)
        .await;

    let response = test_setup.post_subscriptions(&body).await;
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "The API did not send a confirmation email."
    );
}

#[tokio::test]
pub async fn subscribe_sends_a_confirmation_mail_with_a_link() {
    let test_setup = test_utils::create_test_setup().await;

    let body = SubscribeRequest {
        email: String::from("ursula_le_guin@gmail.com"),
        name: String::from("Ursula le Quin"),
    };

    // Set up the mock server and tell it what the request should look like.
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_setup.email_server)
        .await;

    let response = test_setup.post_subscriptions(&body).await;

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Email server returned an error."
    );

    // Get the first intercepted request
    let email_request = &test_setup.email_server.received_requests().await.unwrap()[0];
    // Parse the body as JSON, starting from raw bytes
    let req_body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

    let get_link = |s: &str| {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();
        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
    };

    let html_link = get_link(&req_body["HtmlBody"].as_str().unwrap());
    let text_link = get_link(&req_body["TextBody"].as_str().unwrap());
    // The two links should be identical
    assert_eq!(html_link, text_link);
}
