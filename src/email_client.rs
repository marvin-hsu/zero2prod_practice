use reqwest::Client;
use secrecy::{ExposeSecret, Secret};

use crate::domain::SubscriberEmail;

pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: SubscriberEmail,
    bear_token: Secret<String>,
}

impl EmailClient {
    pub fn new(
        base_url: String,
        sender: SubscriberEmail,
        bear_token: Secret<String>,
        timeout: std::time::Duration,
    ) -> Self {
        let http_client = Client::builder().timeout(timeout).build().unwrap();

        Self {
            http_client: http_client,
            base_url,
            sender,
            bear_token,
        }
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        content_type: &str,
        content: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/v3/mail/send", self.base_url);
        let request_body = SendEmailRequest {
            from: Email {
                email: self.sender.as_ref(),
            },
            personalizations: vec![Personalization {
                to: vec![Email {
                    email: recipient.as_ref(),
                }],
                subject: subject,
            }],
            content: vec![Content {
                type_field: content_type,
                value: content,
            }],
        };
        self.http_client
            .post(&url)
            .bearer_auth(self.bear_token.expose_secret())
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

#[derive(serde::Serialize)]
pub struct SendEmailRequest<'a> {
    pub personalizations: Vec<Personalization<'a>>,
    pub content: Vec<Content<'a>>,
    pub from: Email<'a>,
}

#[derive(serde::Serialize)]
pub struct Personalization<'a> {
    pub to: Vec<Email<'a>>,
    pub subject: &'a str,
}

#[derive(serde::Serialize)]
pub struct Email<'a> {
    pub email: &'a str,
}

#[derive(serde::Serialize)]
pub struct Content<'a> {
    #[serde(rename = "type")]
    pub type_field: &'a str,
    pub value: &'a str,
}

#[cfg(test)]
mod tests {
    use claims::assert_ok;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{faker::internet::en::SafeEmail, Fake};
    use secrecy::Secret;
    use wiremock::matchers::{any, bearer_token, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::{domain::SubscriberEmail, email_client::EmailClient};

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        // Arrange
        let mock_server = MockServer::start().await;
        let token = bear_token();
        let email_client = email_client(mock_server.uri(), token.clone());

        Mock::given(bearer_token(token))
            .and(header("Content-Type", "application/json"))
            .and(path("/v3/mail/send"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = email_client
            .send_email(email(), &subject(), &content_type(), &content())
            .await;

        // Assert
        assert_ok!(outcome);
    }

    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() {
        // Arrange
        let mock_server = MockServer::start().await;
        let token = bear_token();
        let email_client = email_client(mock_server.uri(), token.clone());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = email_client
            .send_email(email(), &subject(), &content_type(), &content())
            .await;

        // Assert
        claims::assert_err!(outcome);
    }

    #[tokio::test]
    async fn send_email_times_out_if_the_server_takes_too_long() {
        // Arrange
        let mock_server = MockServer::start().await;
        let token = bear_token();
        let email_client = email_client(mock_server.uri(), token.clone());

        let response = ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(180));
        Mock::given(any())
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = email_client
            .send_email(email(), &subject(), &content_type(), &content())
            .await;

        // Assert
        claims::assert_err!(outcome);
    }

    fn subject() -> String {
        Sentence(1..2).fake()
    }

    fn content_type() -> String {
        Sentence(1..10).fake()
    }

    fn content() -> String {
        Paragraph(1..10).fake()
    }

    fn email() -> SubscriberEmail {
        SubscriberEmail::parse(SafeEmail().fake()).unwrap()
    }

    fn bear_token() -> String {
        Sentence(1..20).fake()
    }

    fn timeout() -> std::time::Duration {
        std::time::Duration::from_millis(200)
    }

    fn email_client(base_url: String, token: String) -> EmailClient {
        EmailClient::new(base_url, email(), Secret::new(token), timeout())
    }
    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &wiremock::Request) -> bool {
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);

            if let Ok(body) = result {
                body.get("personalizations").unwrap().as_array().unwrap()[0]
                    .get("to")
                    .unwrap()
                    .as_array()
                    .unwrap()[0]
                    .get("email")
                    .is_some()
                    && body.get("personalizations").unwrap().as_array().unwrap()[0]
                        .get("subject")
                        .is_some()
                    && body.get("from").unwrap().get("email").is_some()
                    && body.get("content").unwrap().as_array().unwrap()[0]
                        .get("type")
                        .is_some()
                    && body.get("content").unwrap().as_array().unwrap()[0]
                        .get("value")
                        .is_some()
            } else {
                false
            }
        }
    }
}
