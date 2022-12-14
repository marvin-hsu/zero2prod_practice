use std::net::TcpListener;

#[tokio::test]
async fn health_check_work() {
    // Arrange
    let address = spawn_app();
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(format!("{}/health_check", address))
        .send()
        .await
        .expect("Failed to execute requests.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_return_a_200_for_valid_form_data() {
    // Arrange
    let app_address = spawn_app();
    let client = reqwest::Client::new();
    let body = "name=hsu%20marvin&email=marvin_hsu%40gmail.com";

    // Act
    let response = client
        .post(format!("{}/subscriptions", &app_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute requests.");

    // Assert
    assert_eq!(200, response.status().as_u16())
}

#[tokio::test]
async fn subscribe_return_a_400_when_data_is_missing() {
    // Arrange
    let app_address = spawn_app();
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=hsu%20marvin", "missing the email"),
        ("email=marvin_hsu%40gmail.com", "missing the name"),
        ("", "missing both name the email"),
    ];

    // Act
    for (invalid_body, error_msg) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", &app_address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute requests.");

        // Assert
        assert_eq!(400, response.status().as_u16())
    }
}

fn spawn_app() -> String {
    let listener = TcpListener::bind("0.0.0.0:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let server = zero2prod_practice::run(listener).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    format!("http://0.0.0:{}", port)
}
