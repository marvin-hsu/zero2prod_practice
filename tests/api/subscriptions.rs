use crate::helpers::spawn_app;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn subscribe_return_a_200_for_valid_form_data() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=hsu%20marvin&email=marvin_hsu%40gmail.com";
    Mock::given(path("/v3/mail/send"))
        .and(method("Post"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act
    let response = app.post_subscriptions(body.into()).await;

    // Assert
    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_persists_the_new_subscriber() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=hsu%20marvin&email=marvin_hsu%40gmail.com";
    Mock::given(path("/v3/mail/send"))
        .and(method("Post"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act
    app.post_subscriptions(body.into()).await;

    // Assert
    let saved = sqlx::query!("Select email, name, status FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "marvin_hsu@gmail.com");
    assert_eq!(saved.name, "hsu marvin");
    assert_eq!(saved.status, "pending_confirmation");
}

#[tokio::test]
async fn subscribe_return_a_400_when_data_is_missing() {
    // Arrange
    let app = spawn_app().await;
    let test_cases = vec![
        ("name=hsu%20marvin", "missing the email"),
        ("email=marvin_hsu%40gmail.com", "missing the name"),
        ("", "missing both name the email"),
    ];

    // Act
    for (invalid_body, _error_msg) in test_cases {
        let response = app.post_subscriptions(invalid_body.into()).await;

        // Assert
        assert_eq!(400, response.status().as_u16())
    }
}

#[tokio::test]
async fn subscribe_return_a_400_when_fields_are_present_but_empty() {
    // Arrange
    let app = spawn_app().await;
    let test_case = vec![
        ("name=&email=marvin_hsu%40gmail.com", "empty name"),
        ("name=marvinhsu&email=", "empty name"),
        (
            "name=marvinhsu&email=definitely-not-an-email",
            "invalid email",
        ),
    ];

    for (body, description) in test_case {
        // Act
        let response = app.post_subscriptions(body.into()).await;

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 400 Bad Request when the payload was {}",
            description
        )
    }
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=hsu%20marvin&email=marvin_hsu%40gmail.com";

    Mock::given(path("/v3/mail/send"))
        .and(method("Post"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act
    app.post_subscriptions(body.into()).await;

    // Assert
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_with_a_link() {
    let app = spawn_app().await;
    let body = "name=hsu%20marvin&email=marvin_hsu%40gmail.com";

    Mock::given(path("/v3/mail/send"))
        .and(method("Post"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;

    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();
    let content = body.get("content").unwrap().as_array().unwrap()[0]
        .get("value")
        .unwrap()
        .to_string();

    let links: Vec<_> = linkify::LinkFinder::new()
        .links(&content)
        .filter(|l| *l.kind() == linkify::LinkKind::Url)
        .collect();

    assert_eq!(links.len(), 1);
}
