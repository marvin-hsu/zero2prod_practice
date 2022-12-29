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

    let saved = sqlx::query!("Select email, name FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "marvin_hsu@gmail.com");
    assert_eq!(saved.name, "hsu marvin")
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
