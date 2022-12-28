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
    pub fn new(base_url: String, sender: SubscriberEmail, bear_token: Secret<String>) -> Self {
        Self {
            http_client: Client::new(),
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
                email: self.sender.as_ref().to_owned(),
            },
            personalizations: vec![Personalization {
                to: vec![Email {
                    email: recipient.as_ref().to_owned(),
                }],
                subject: subject.to_owned(),
            }],
            content: vec![Content {
                type_field: content_type.to_owned(),
                value: content.to_owned(),
            }],
        };
        self.http_client
            .post(&url)
            .bearer_auth(self.bear_token.expose_secret())
            .json(&request_body)
            .send()
            .await?;
        Ok(())
    }
}

#[derive(serde::Serialize)]
pub struct SendEmailRequest {
    pub personalizations: Vec<Personalization>,
    pub content: Vec<Content>,
    pub from: Email,
}

#[derive(serde::Serialize)]
pub struct Personalization {
    pub to: Vec<Email>,
    pub subject: String,
}

#[derive(serde::Serialize)]
pub struct Email {
    pub email: String,
}

#[derive(serde::Serialize)]
pub struct Content {
    #[serde(rename = "type")]
    pub type_field: String,
    pub value: String,
}

#[cfg(test)]
mod tests {
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::Faker;
    use fake::{faker::internet::en::SafeEmail, Fake};
    use secrecy::Secret;
    use wiremock::matchers::{bearer_token, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::{domain::SubscriberEmail, email_client::EmailClient};

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        // Arrange
        let mock_server = MockServer::start().await;
        let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let token: String = Faker.fake();
        let email_client = EmailClient::new(mock_server.uri(), sender, Secret::new(token.clone()));

        Mock::given(bearer_token(token))
            .and(header("Content-Type", "application/json"))
            .and(path("/v3/mail/send"))
            .and(method("POST"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content_type: String = Paragraph(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        // Act
        let _ = email_client
            .send_email(subscriber_email, &subject, &content_type, &content)
            .await;
    }
}
