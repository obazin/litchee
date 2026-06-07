//! Integration tests for the Messaging API.

use litchee::LichessClient;
use wiremock::matchers::{body_string_contains, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> LichessClient {
    LichessClient::builder()
        .base_url(&server.uri().parse().expect("mock uri is a valid url"))
        .token("test-token")
        .build()
        .expect("client builds")
}

#[tokio::test]
async fn send_posts_the_message_text() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/inbox/bobby"))
        .and(body_string_contains("text=hello"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;

    client(&server)
        .messaging()
        .send("bobby", "hello")
        .await
        .unwrap();
}
