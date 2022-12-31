use reqwest::Url;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::spawn_app;

#[tokio::test]
async fn confirmations_without_token_are_rejected_with_a_400() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = reqwest::get(format!("{}/subscriptions/confirm", app.address))
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn the_link_return_by_subscribe_returns_a_200_if_called() {
    let app = spawn_app().await;
    let body = "name=marvinhsu&email=marvinhsu@gmail.com";

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
    let mut confirmation_link = Url::parse(links[0].as_str()).unwrap();
    assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
    confirmation_link.set_port(Some(app.port)).unwrap();

    let response = reqwest::get(confirmation_link).await.unwrap();

    assert_eq!(response.status().as_u16(), 200);
}
