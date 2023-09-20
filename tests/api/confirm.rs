use crate::subscribe;
use crate::test_utils;
use axum::http::StatusCode;
use axum_test_helper::TestResponse;
use url::Url;
use wiremock::matchers::any;
use wiremock::{Mock, ResponseTemplate};

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

pub fn get_confirmation_links(email_request: &wiremock::Request) -> String {
    let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();
    // Extract the link from one of the request fields.
    let get_link = |s: &str| {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();
        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
        // let raw_link = links[0].as_str().to_owned();
        // let mut confirmation_link = reqwest::Url::parse(&raw_link).unwrap();
        // // Let's make sure we don't call random APIs on the web
        // assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
        // confirmation_link.set_port(Some(self.port)).unwrap();
        // confirmation_link
    };
    let html = get_link(&body["HtmlBody"].as_str().unwrap());
    let plain_text = get_link(&body["TextBody"].as_str().unwrap());
    // The two links should be identical
    assert_eq!(html, plain_text);
    plain_text
}

#[tokio::test]
async fn confirmations_with_invalid_token_are_rejected_with_a_401() {
    // Arrange
    let test_setup = test_utils::create_test_setup().await;
    let query = CorrectQueryParams {
        token: "gibberish".to_string(),
    };

    // Act
    let response = test_setup.post_confirm(&query).await;

    // Assert
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
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
pub async fn subscribe_sends_a_confirmation_link() {
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
    get_confirmation_links(email_request);
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
    let conf_link = get_confirmation_links(email_request);

    // TestClient takes just the route as the 'address', so strip away the base url
    println!("Confirmation link: {}", conf_link);
    let route = extract_route(&conf_link);
    println!("Route: {:?}", route);

    // let route_with_token = format!("{}?token=somerandomstring", route);

    let response = test_setup
        .client
        .post(&route) //_with_token)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send()
        .await;

    assert_eq!(response.status(), StatusCode::OK)
}

#[tokio::test]
pub async fn subscribe_confirmation_link_confirms_user() {
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
    let conf_link = get_confirmation_links(email_request);

    // TestClient takes just the route as the 'address', so strip away the base url
    println!("Confirmation link: {}", conf_link);
    let route = extract_route(&conf_link);
    println!("Route: {:?}", route);

    // let route_with_token = format!("{}?token=somerandomstring", route);

    let response = test_setup
        .client
        .post(&route)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send()
        .await;

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "/confirm did not respond with 200 OK."
    );

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&test_setup.pg_pool)
        .await
        .expect("Failed to fetch saved subscription.");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "Ursula le Quin");
    assert_eq!(saved.status, "confirmed");
}

fn extract_route(url_str: &str) -> String {
    let url = match Url::parse(url_str) {
        Ok(u) => u,
        Err(_) => "".try_into().expect("Failed!"),
    };
    let mut route: String;
    if let Some(path_segments) = url.path_segments() {
        let route_segments = path_segments.collect::<Vec<_>>();
        route = route_segments.join("/");
        if url.query().is_some() {
            route = format!("{}?{}", route, url.query().unwrap());
        };
    } else {
        route = "".to_string();
    }
    format!("/{}", route)
}
