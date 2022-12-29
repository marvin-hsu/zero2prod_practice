use crate::helpers::spawn_app;

#[tokio::test]
async fn subscribe_return_a_200_for_valid_form_data() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let body = "name=hsu%20marvin&email=marvin_hsu%40gmail.com";

    // Act
    let response = client
        .post(format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute requests.");

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
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=hsu%20marvin", "missing the email"),
        ("email=marvin_hsu%40gmail.com", "missing the name"),
        ("", "missing both name the email"),
    ];

    // Act
    for (invalid_body, _error_msg) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute requests.");

        // Assert
        assert_eq!(400, response.status().as_u16())
    }
}

#[tokio::test]
async fn subscribe_return_a_400_when_fields_are_present_but_empty() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
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
        let response = client
            .post(format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request");

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 400 Bad Request when the payload was {}",
            description
        )
    }
}
