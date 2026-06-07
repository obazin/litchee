//! Integration tests for the Broadcasts API.

use futures_util::StreamExt;
use litchee::LichessClient;
use wiremock::matchers::{body_string_contains, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> LichessClient {
    LichessClient::builder()
        .base_url(&server.uri().parse().expect("mock uri is a valid url"))
        .build()
        .expect("client builds")
}

#[tokio::test]
async fn official_streams_broadcasts() {
    let server = MockServer::start().await;
    let body = concat!(
        r#"{"tour":{"id":"a","name":"One"},"rounds":[{"id":"r1","name":"R1"}]}"#,
        "\n",
        r#"{"tour":{"id":"b","name":"Two"},"rounds":[]}"#,
        "\n",
    );
    Mock::given(method("GET"))
        .and(path("/api/broadcast"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let stream = client(&server).broadcasts().official().await.unwrap();
    let broadcasts: Vec<_> = stream.collect().await;

    assert_eq!(broadcasts.len(), 2);
    assert_eq!(broadcasts[0].as_ref().unwrap().tour.name, "One");
}

#[tokio::test]
async fn get_tournament_returns_rounds() {
    let server = MockServer::start().await;
    let body = r#"{"tour":{"id":"abc","name":"World Champ","slug":"wc"},
        "rounds":[{"id":"r1","name":"Round 1","rated":true,"finished":false}]}"#;
    Mock::given(method("GET"))
        .and(path("/api/broadcast/abc"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let broadcast = client(&server)
        .broadcasts()
        .get_tournament("abc")
        .await
        .unwrap();

    assert_eq!(broadcast.rounds[0].id, "r1");
}

#[tokio::test]
async fn round_pgn_returns_text() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/broadcast/round/r1.pgn"))
        .respond_with(ResponseTemplate::new(200).set_body_string("[Event \"Round 1\"]\n\n1. e4 *"))
        .mount(&server)
        .await;

    let pgn = client(&server).broadcasts().round_pgn("r1").await.unwrap();

    assert!(pgn.contains("Round 1"));
}

#[tokio::test]
async fn reset_round_posts_to_reset() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/broadcast/round/r1/reset"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;

    client(&server)
        .broadcasts()
        .reset_round("r1")
        .await
        .unwrap();
}

#[tokio::test]
async fn top_returns_active_and_upcoming() {
    let server = MockServer::start().await;
    let body = r#"{"active":[{"tour":{"id":"a","name":"A"},"rounds":[]}],"upcoming":[],"past":{}}"#;
    Mock::given(method("GET"))
        .and(path("/api/broadcast/top"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;
    let top = client(&server).broadcasts().top().await.unwrap();
    assert_eq!(top.active[0].tour.id, "a");
}

#[tokio::test]
async fn search_paginates() {
    let server = MockServer::start().await;
    let body = r#"{"currentPage":1,"maxPerPage":20,"currentPageResults":[],"previousPage":null,"nextPage":2}"#;
    Mock::given(method("GET"))
        .and(path("/api/broadcast/search"))
        .and(query_param("q", "wch"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;
    let page = client(&server).broadcasts().search("wch", 1).await.unwrap();
    assert_eq!(page.next_page, Some(2));
}

#[tokio::test]
async fn create_tour_posts_to_new() {
    let server = MockServer::start().await;
    let body = r#"{"tour":{"id":"t1","name":"My Tour"},"rounds":[]}"#;
    Mock::given(method("POST"))
        .and(path("/broadcast/new"))
        .and(body_string_contains("name=My+Tour"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;
    let tour = client(&server)
        .broadcasts()
        .create_tour("My Tour")
        .info("desc")
        .send()
        .await
        .unwrap();
    assert_eq!(tour.tour.id, "t1");
}

#[tokio::test]
async fn push_pgn_posts_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/broadcast/round/r1/push"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"games":[]}"#))
        .mount(&server)
        .await;
    let result = client(&server)
        .broadcasts()
        .push_pgn("r1", "1. e4 *")
        .await
        .unwrap();
    assert!(result.data.contains_key("games"));
}

#[tokio::test]
async fn players_returns_entries() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/broadcast/t1/players"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(r#"[{"name":"Carlsen","fideId":1503014}]"#),
        )
        .mount(&server)
        .await;
    let players = client(&server).broadcasts().players("t1").await.unwrap();
    assert_eq!(players[0].name.as_deref(), Some("Carlsen"));
}
