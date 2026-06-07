//! Integration tests for the Opening Explorer API (explorer host routing).

use futures_util::StreamExt;
use litchee::LichessClient;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Points the *explorer* host at the mock server.
fn client(server: &MockServer) -> LichessClient {
    LichessClient::builder()
        .opening_explorer_url(&server.uri().parse().expect("mock uri is a valid url"))
        .build()
        .expect("client builds")
}

const RESULT: &str = r#"{"opening":{"eco":"B01","name":"Scandinavian"},
    "white":100,"draws":40,"black":60,
    "moves":[{"uci":"e2e4","san":"e4","white":50,"draws":20,"black":30}]}"#;

#[tokio::test]
async fn masters_uses_the_explorer_host() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/masters"))
        .and(query_param("fen", "startpos"))
        .respond_with(ResponseTemplate::new(200).set_body_string(RESULT))
        .mount(&server)
        .await;

    let result = client(&server)
        .opening_explorer()
        .masters("startpos", None)
        .await
        .unwrap();

    assert_eq!(result.white, 100);
    assert_eq!(result.moves[0].san, "e4");
}

#[tokio::test]
async fn lichess_sends_play_moves() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/lichess"))
        .and(query_param("play", "e2e4,e7e5"))
        .respond_with(ResponseTemplate::new(200).set_body_string(RESULT))
        .mount(&server)
        .await;

    let result = client(&server)
        .opening_explorer()
        .lichess("startpos", Some("e2e4,e7e5"))
        .await
        .unwrap();

    assert_eq!(result.black, 60);
}

#[tokio::test]
async fn masters_pgn_returns_text() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/masters/pgn/abcd"))
        .respond_with(ResponseTemplate::new(200).set_body_string("[Event \"x\"]\n\n1. e4 *"))
        .mount(&server)
        .await;

    let pgn = client(&server)
        .opening_explorer()
        .masters_pgn("abcd")
        .await
        .unwrap();

    assert!(pgn.contains("1. e4"));
}

#[tokio::test]
async fn player_streams_results() {
    let server = MockServer::start().await;
    let body = concat!(
        r#"{"white":1,"draws":0,"black":0,"moves":[]}"#,
        "\n",
        r#"{"white":2,"draws":1,"black":0,"moves":[]}"#,
        "\n",
    );
    Mock::given(method("GET"))
        .and(path("/player"))
        .and(query_param("player", "bobby"))
        .and(query_param("color", "white"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;
    let stream = client(&server)
        .opening_explorer()
        .player("bobby", "white", "startpos", None)
        .await
        .unwrap();
    let results: Vec<_> = stream.collect().await;
    assert_eq!(results.len(), 2);
}
