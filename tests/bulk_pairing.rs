//! Integration tests for the Bulk Pairing API.

use futures_util::StreamExt;
use litchee::LichessClient;
use litchee::model::GameExportOptions;
use wiremock::matchers::{body_string_contains, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> LichessClient {
    LichessClient::builder()
        .base_url(&server.uri().parse().expect("mock uri is a valid url"))
        .token("test-token")
        .build()
        .expect("client builds")
}

const PAIRING: &str = r#"{"id":"RV","games":[{"id":"g","white":"a","black":"b"}],
    "variant":"standard","clock":{"limit":300,"increment":0},
    "pairAt":1,"pairedAt":null,"rated":false,"startClocksAt":2,"scheduledAt":3}"#;

#[tokio::test]
async fn list_returns_pairings() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/bulk-pairing"))
        .respond_with(ResponseTemplate::new(200).set_body_string(format!("[{PAIRING}]")))
        .mount(&server)
        .await;

    let pairings = client(&server).bulk_pairing().list().await.unwrap();

    assert_eq!(pairings.len(), 1);
    assert_eq!(pairings[0].id, "RV");
}

#[tokio::test]
async fn get_returns_a_pairing() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/bulk-pairing/RV"))
        .respond_with(ResponseTemplate::new(200).set_body_string(PAIRING))
        .mount(&server)
        .await;

    let pairing = client(&server).bulk_pairing().get("RV").await.unwrap();

    assert_eq!(pairing.id, "RV");
}

#[tokio::test]
async fn games_streams_games() {
    let server = MockServer::start().await;
    let body = "{\"id\":\"g1\"}\n{\"id\":\"g2\"}\n";
    Mock::given(method("GET"))
        .and(path("/api/bulk-pairing/RV/games"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let stream = client(&server)
        .bulk_pairing()
        .games("RV")
        .stream()
        .await
        .unwrap();
    let games: Vec<_> = stream.collect().await;

    assert_eq!(games.len(), 2);
    assert_eq!(games[0].as_ref().unwrap().id, "g1");
}

#[tokio::test]
async fn games_sends_export_params() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/bulk-pairing/RV/games"))
        .and(query_param("clocks", "true"))
        .and(query_param("evals", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_string("{\"id\":\"g1\"}\n"))
        .mount(&server)
        .await;
    let stream = client(&server)
        .bulk_pairing()
        .games("RV")
        .export(GameExportOptions::default().clocks(true).evals(true))
        .stream()
        .await
        .unwrap();
    let games: Vec<_> = stream.collect().await;
    assert_eq!(games.len(), 1);
}

#[tokio::test]
async fn games_pgn_sets_accept_header() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/bulk-pairing/RV/games"))
        .and(header("accept", "application/x-chess-pgn"))
        .respond_with(ResponseTemplate::new(200).set_body_string("[Event \"x\"]\n\n1. e4 *"))
        .mount(&server)
        .await;
    let pgn = client(&server)
        .bulk_pairing()
        .games("RV")
        .pgn()
        .await
        .unwrap();
    assert!(pgn.contains("1. e4"));
}

#[tokio::test]
async fn create_posts_players_and_clock() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/bulk-pairing"))
        .and(body_string_contains("players=t1%3At2"))
        .and(body_string_contains("clock.limit=300"))
        .respond_with(ResponseTemplate::new(200).set_body_string(PAIRING))
        .mount(&server)
        .await;

    let pairing = client(&server)
        .bulk_pairing()
        .create("t1:t2")
        .clock(300, 0)
        .rated(false)
        .send()
        .await
        .unwrap();

    assert_eq!(pairing.id, "RV");
}

#[tokio::test]
async fn delete_removes_a_pairing() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/api/bulk-pairing/RV"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;

    client(&server).bulk_pairing().delete("RV").await.unwrap();
}

#[tokio::test]
async fn start_clocks_posts_to_the_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/bulk-pairing/RV/start-clocks"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;

    client(&server)
        .bulk_pairing()
        .start_clocks("RV")
        .await
        .unwrap();
}
