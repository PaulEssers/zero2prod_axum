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
    pub fn new(base_url: String, sender: ValidEmail, authorization_token: String) -> Self {
        Self {
            http_client: Client::new(),
            base_url,
            sender,
            authorization_token,
        }
    }

    pub async fn send_email(
        &self,
        recipient: ValidEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), Error> {
        let base_url = reqwest::Url::parse(&self.base_url)?;
        let url = reqwest::Url::join(&base_url, "/email")?;

        let request_body = SendEmailRequest {
            from: self.sender.as_str().to_owned(),
            to: recipient.as_str().to_owned(),
            subject: subject.to_owned(),
            html_body: html_content.to_owned(),
            text_body: text_content.to_owned(),
        };

        let _ = self
            .http_client
            .post(url)
            .header("X-Postmark-Server-Token", &self.authorization_token)
            .json(&request_body)
            .send()
            .await?;
        Ok(())
    }
}

#[derive(serde::Serialize)]
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
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use wiremock::matchers::any;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        // Arrange
        let mock_server = MockServer::start().await;
        let fake_sender: String = SafeEmail().fake();
        //.expect("Failed to create fake email.");
        let sender = ValidEmail::new(&fake_sender).expect("Fake email was invalid");
        let token = Faker.fake();
        let email_client = EmailClient::new(mock_server.uri(), sender, token);
        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;
        let fake_sub: String = SafeEmail().fake();
        //.expect("Failed to create fake email.");
        let subscriber_email = ValidEmail::new(&fake_sub).expect("Fake email was invalid");
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();
        // Act
        let _ = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;

        // Assert -> when mock_server goes out of scope, it asserts it has received the request
    }
}