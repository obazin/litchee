//! Integration tests for the Tablebase API (separate host routing).

use litchee::LichessClient;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Points the *tablebase* host at the mock server, leaving the default host
/// untouched — this verifies multi-host routing.
fn client(server: &MockServer) -> LichessClient {
    LichessClient::builder()
        .tablebase_url(&server.uri().parse().expect("mock uri is a valid url"))
        .build()
        .expect("client builds")
}

#[tokio::test]
async fn standard_lookup_uses_the_tablebase_host() {
    let server = MockServer::start().await;
    let body = r#"{"category":"win","dtz":1,"checkmate":false,
        "moves":[{"uci":"h7h8q","san":"h8=Q+","category":"loss"}]}"#;
    Mock::given(method("GET"))
        .and(path("/standard"))
        .and(query_param("fen", "4k3/6KP/8/8/8/8/7p/8 w - - 0 1"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let position = client(&server)
        .tablebase()
        .standard("4k3/6KP/8/8/8/8/7p/8 w - - 0 1", None)
        .await
        .unwrap();

    assert_eq!(position.moves.len(), 1);
    assert_eq!(position.dtz, Some(1));
}

#[tokio::test]
async fn standard_sends_dtc_query() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/standard"))
        .and(query_param("dtc", "always"))
        .and(query_param("fen", "8/8/8/8/8/8/8/K6k w - - 0 1"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(r#"{"category":"draw","moves":[]}"#),
        )
        .mount(&server)
        .await;

    let position = client(&server)
        .tablebase()
        .standard("8/8/8/8/8/8/8/K6k w - - 0 1", Some("always"))
        .await
        .unwrap();

    assert!(position.moves.is_empty());
}

#[tokio::test]
async fn atomic_lookup_hits_the_variant_path() {
    let server = MockServer::start().await;
    let body = r#"{"category":"win","moves":[{"uci":"e2e4","san":"e4","category":"loss"}]}"#;
    Mock::given(method("GET"))
        .and(path("/atomic"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let position = client(&server).tablebase().atomic("fen").await.unwrap();

    assert_eq!(position.moves.len(), 1);
}

#[tokio::test]
async fn antichess_lookup_hits_the_variant_path() {
    let server = MockServer::start().await;
    let body = r#"{"category":"draw","moves":[]}"#;
    Mock::given(method("GET"))
        .and(path("/antichess"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let position = client(&server).tablebase().antichess("fen").await.unwrap();

    assert!(position.moves.is_empty());
}
