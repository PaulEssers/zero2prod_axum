use crate::error::Error;
use reqwest::Client;
use validator::validate_email;

#[derive(Debug, Clone)]
pub struct ValidEmail(String);
impl ValidEmail {
    pub fn new(email: &str) -> Result<Self, String> {
        if validate_email(email) {
            Ok(ValidEmail(email.to_string()))
        } else {
            Err("Not a valid email!".to_string())
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: ValidEmail,
    authorization_token: String,
}

impl EmailClient {
    pub fn new(
        base_url: String,
        sender: ValidEmail,
        authorization_token: String,
        timeout: std::time::Duration,
    ) -> Self {
        let http_client = Client::builder().timeout(timeout).build().unwrap();
        Self {
            http_client,
            base_url,
            sender,
            authorization_token,
        }
    }

    pub async fn send_email<'a>(
        &self,
        recipient: &'a ValidEmail,
        subject: &'a str,
        html_content: &'a str,
        text_content: &'a str,
    ) -> Result<(), Error> {
        let base_url = reqwest::Url::parse(&self.base_url)?;
        let url = reqwest::Url::join(&base_url, "/email")?;

        let request_body = SendEmailRequest {
            from: self.sender.as_str().to_string(),
            to: recipient.as_str().to_string(),
            subject: subject.to_string(),
            html_body: html_content.to_string(),
            text_body: text_content.to_string(),
        };

        let _ = self
            .http_client
            .post(url)
            .header("X-Postmark-Server-Token", &self.authorization_token)
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest {
    from: String,
    to: String,
    subject: String,
    html_body: String,
    text_body: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::email_client::EmailClient;
    use claim::{assert_err, assert_ok};
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use wiremock::matchers::{any, body_json_schema, header, header_exists, method, path};
    use wiremock::Request;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    // Custom check for use in wiremock
    struct SendEmailBodyMatcher;
    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            // Try to parse the body as a JSON value
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);
            if let Ok(body) = result {
                // Check that all the mandatory fields are populated
                // without inspecting the field values
                body.get("From").is_some()
                    && body.get("To").is_some()
                    && body.get("Subject").is_some()
                    && body.get("HtmlBody").is_some()
                    && body.get("TextBody").is_some()
            } else {
                // If parsing failed, do not match the request
                false
            }
        }
    }

    // functions for generating fake email components.
    fn email() -> ValidEmail {
        let fake_sender: String = SafeEmail().fake();
        ValidEmail::new(&fake_sender).expect("Fake email was invalid")
    }

    fn token() -> String {
        Faker.fake()
    }

    fn subject() -> String {
        Sentence(1..2).fake()
    }

    fn content() -> String {
        Paragraph(1..10).fake()
    }

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = EmailClient::new(
            mock_server.uri(),
            email(),
            token(),
            std::time::Duration::from_millis(200),
        );

        // Set up the mock server and tell it what the request should look like.
        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .and(body_json_schema::<SendEmailRequest>)
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = email_client
            .send_email(&email(), &subject(), &content(), &content())
            .await;

        // Assert
        assert_ok!(outcome);
        // when mock_server goes out of scope here, it asserts it has received the 1 expected request.
    }

    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = EmailClient::new(
            mock_server.uri(),
            email(),
            Faker.fake(),
            std::time::Duration::from_millis(200),
        );

        Mock::given(any())
            // Not a 200 anymore!
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;
        // Act
        let outcome = email_client
            .send_email(&email(), &subject(), &content(), &content())
            .await;
        // Assert
        assert_err!(outcome);
    }

    #[tokio::test]
    async fn send_email_times_out_if_the_server_takes_too_long() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = EmailClient::new(
            mock_server.uri(),
            email(),
            Faker.fake(),
            std::time::Duration::from_millis(200),
        );

        let response = ResponseTemplate::new(200)
            // 3 minutes!
            .set_delay(std::time::Duration::from_secs(180));
        Mock::given(any())
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;
        // Act
        let outcome = email_client
            .send_email(&email(), &subject(), &content(), &content())
            .await;
        // Assert
        assert_err!(outcome);
    }
}
