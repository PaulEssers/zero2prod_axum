use crate::subscribe;
use crate::test_utils;
use axum::http::StatusCode;
use axum_test_helper::TestResponse;
use url::Url;
use wiremock::matchers::{any, method, path};
use wiremock::{Mock, ResponseTemplate};
use zero2prod::error::Error;

// Trait that flags a struct as valid for use in posting queries
pub trait QueryParams {}

#[derive(Debug, serde::Serialize)]
pub struct CorrectQueryParams {
    token: String,
}
impl QueryParams for CorrectQueryParams {}

impl test_utils::TestSetup {
    pub async fn post_confirm<T>(&self, query: &T) -> TestResponse
    where
        T: QueryParams + serde::Serialize,
    {
        let query_string = serde_urlencoded::to_string(query).unwrap();
        let url = format!("/confirm?{}", query_string);
        println!("Query url = {}", &url);

        self.client
            .post(&url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .send()
            .await
    }
}

#[tokio::test]
async fn confirmations_with_token_are_accepted_with_a_200() {
    // Arrange
    let test_setup = test_utils::create_test_setup().await;
    let query = CorrectQueryParams {
        token: "gibberish".to_string(),
    };

    // Act
    let response = test_setup.post_confirm(&query).await;

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
}

#[derive(Debug, serde::Serialize)]
pub struct EmptyQueryParams {}
impl QueryParams for EmptyQueryParams {}

#[tokio::test]
async fn confirmations_without_token_are_rejected_with_a_400() {
    // Arrange
    let test_setup = test_utils::create_test_setup().await;
    let query = EmptyQueryParams {};

    // Act
    let response = test_setup.post_confirm(&query).await;

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
pub async fn subscribe_confirmation_link_works() {
    let test_setup = test_utils::create_test_setup().await;

    let body = subscribe::SubscribeRequest {
        email: String::from("ursula_le_guin@gmail.com"),
        name: String::from("Ursula le Quin"),
    };

    // Set up the mock server and tell it what the request should look like.
    Mock::given(any())
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

    // TestClient takes just the route as the 'address', so strip away the base url
    println!("Confirmation link: {}", html_link);
    let route = extract_route(&html_link);
    println!("Route: {:?}", route);

    let response = test_setup
        .client
        .post(&route)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send()
        .await;
}

fn extract_route(url_str: &str) -> String {
    let url = match Url::parse(url_str) {
        Ok(u) => u,
        Err(_) => "".try_into().expect("Failed!"),
    };
    let route: String;
    if let Some(path_segments) = url.path_segments() {
        route = path_segments.collect::<Vec<_>>().join("/");
    } else {
        route = "".to_string();
    }
    format!("/{}", route)
}
