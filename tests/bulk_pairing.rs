//! Integration tests for the Bulk Pairing API.

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
