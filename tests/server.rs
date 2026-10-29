//! Integration tests for the Server API.

// CLAUDE.md rule 2 scopes the 20-line function cap to src/; integration
// tests sit outside that boundary, so a linear arrange/assert case may
// run long rather than be split for the sake of the count.
#![allow(clippy::too_many_lines)]

use litchee::LichessClient;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> LichessClient {
    LichessClient::builder()
        .base_url(&server.uri().parse().expect("mock uri is a valid url"))
        .build()
        .expect("client builds")
}

#[tokio::test]
async fn version_returns_the_deployed_build() {
    let server = MockServer::start().await;
    let body = r#"{"server":{"date":"2026-10-28T07:25:58Z","commit":"b551752d2a0e7a63459d9812825d95a633a98737"}}"#;
    Mock::given(method("GET"))
        .and(path("/api/version"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let version = client(&server).server().version().await.unwrap();

    assert_eq!(version.server.date, "2026-10-28T07:25:58Z");
    assert_eq!(
        version.server.commit,
        "b551752d2a0e7a63459d9812825d95a633a98737"
    );
}
