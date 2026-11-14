//! Integration tests for the `User-Agent` header, which Lichess requires on
//! every request.

// CLAUDE.md rule 2 scopes the 20-line function cap to src/; integration
// tests sit outside that boundary, so a linear arrange/assert case may
// run long rather than be split for the sake of the count.
#![allow(clippy::too_many_lines)]

use litchee::LichessClient;
use litchee::LichessClientBuilder;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const DEFAULT_USER_AGENT: &str = concat!("litchee/", env!("CARGO_PKG_VERSION"));

/// Mounts `GET /api/version`, answering only when the expected `User-Agent`
/// arrives, so a missing or wrong header fails the request with a 404.
async fn expect_user_agent(server: &MockServer, user_agent: &str) {
    Mock::given(method("GET"))
        .and(path("/api/version"))
        .and(header("user-agent", user_agent))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(include_str!("fixtures/server_version.json")),
        )
        .expect(1)
        .mount(server)
        .await;
}

fn builder(server: &MockServer) -> LichessClientBuilder {
    LichessClient::builder().base_url(&server.uri().parse().expect("mock uri is a valid url"))
}

#[tokio::test]
async fn built_in_client_sends_the_default_user_agent() {
    let server = MockServer::start().await;
    expect_user_agent(&server, DEFAULT_USER_AGENT).await;

    let client = builder(&server).build().unwrap();

    client.server().version().await.unwrap();
}

#[tokio::test]
async fn built_in_client_sends_a_custom_user_agent() {
    let server = MockServer::start().await;
    expect_user_agent(&server, "my-bot (contact@example.org)").await;

    let client = builder(&server)
        .user_agent("my-bot (contact@example.org)")
        .build()
        .unwrap();

    client.server().version().await.unwrap();
}

#[tokio::test]
async fn custom_http_client_without_a_user_agent_sends_the_default() {
    let server = MockServer::start().await;
    expect_user_agent(&server, DEFAULT_USER_AGENT).await;

    let client = builder(&server)
        .http_client(reqwest::Client::new())
        .build()
        .unwrap();

    client.server().version().await.unwrap();
}

#[tokio::test]
async fn custom_http_client_sends_the_configured_user_agent() {
    let server = MockServer::start().await;
    expect_user_agent(&server, "my-bot (https://example.org)").await;

    let client = builder(&server)
        .http_client(reqwest::Client::new())
        .user_agent("my-bot (https://example.org)")
        .build()
        .unwrap();

    client.server().version().await.unwrap();
}

#[tokio::test]
async fn custom_clients_own_user_agent_is_replaced_by_the_default() {
    let server = MockServer::start().await;
    expect_user_agent(&server, DEFAULT_USER_AGENT).await;
    let http = reqwest::Client::builder()
        .user_agent("set-on-reqwest")
        .build()
        .unwrap();

    let client = builder(&server).http_client(http).build().unwrap();

    client.server().version().await.unwrap();
}

#[tokio::test]
async fn configured_user_agent_overrides_the_custom_clients_own() {
    let server = MockServer::start().await;
    expect_user_agent(&server, "my-bot").await;
    let http = reqwest::Client::builder()
        .user_agent("set-on-reqwest")
        .build()
        .unwrap();

    let client = builder(&server)
        .http_client(http)
        .user_agent("my-bot")
        .build()
        .unwrap();

    client.server().version().await.unwrap();
}
