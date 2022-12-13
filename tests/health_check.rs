//#[tokio::test]
//async fn health_check_work() {
// // Arrange
// spawn_app();
// let client = reqwest::Client::new();

// // Act
// let response = client
//        .get("http://0.0.0.0:8000/health_check")
//        .send()
//        .await
//        .expect("Failed to execute requests.");

//    // Assert
//    assert!(response.status().is_success());
//    assert_eq!(Some(0), response.content_length());
//}

// fn spawn_app() {
// let server = zero2prod_practice::run().expect("Failed to bind address");

// let _ = tokio::spawn(server);
// }
