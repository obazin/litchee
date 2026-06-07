//! Integration tests for the Swiss Tournaments API.

use futures_util::StreamExt;
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
async fn get_returns_a_swiss() {
    let server = MockServer::start().await;
    let body = r#"{"id":"abc","name":"Weekly","nbRounds":7,"round":2,"status":"started"}"#;
    Mock::given(method("GET"))
        .and(path("/api/swiss/abc"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let swiss = client(&server).swiss().get("abc").await.unwrap();

    assert_eq!(swiss.nb_rounds, Some(7));
}

#[tokio::test]
async fn create_posts_to_team_path_with_clock() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/swiss/new/coders"))
        .and(body_string_contains("clock.limit=300"))
        .and(body_string_contains("nbRounds=7"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"id":"new1"}"#))
        .mount(&server)
        .await;

    let swiss = client(&server)
        .swiss()
        .create("coders", 300, 0, 7)
        .name("Weekly")
        .rated(true)
        .send()
        .await
        .unwrap();

    assert_eq!(swiss.id, "new1");
}

#[tokio::test]
async fn join_posts_to_the_join_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/swiss/abc/join"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;

    client(&server).swiss().join("abc").await.unwrap();
}

#[tokio::test]
async fn trf_returns_text() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/swiss/abc.trf"))
        .respond_with(ResponseTemplate::new(200).set_body_string("012 Lichess Swiss\n"))
        .mount(&server)
        .await;

    let trf = client(&server).swiss().trf("abc").await.unwrap();

    assert!(trf.contains("Lichess Swiss"));
}

#[tokio::test]
async fn results_streams_rows() {
    let server = MockServer::start().await;
    let body = concat!(
        r#"{"rank":1,"points":5.5,"username":"A","rating":2400}"#,
        "\n",
        r#"{"rank":2,"points":5.0,"username":"B","rating":2350}"#,
        "\n",
    );
    Mock::given(method("GET"))
        .and(path("/api/swiss/abc/results"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let stream = client(&server).swiss().results("abc").await.unwrap();
    let results: Vec<_> = stream.collect().await;

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].as_ref().unwrap().username, "A");
}

#[tokio::test]
async fn edit_posts_to_edit_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/swiss/abc/edit"))
        .and(body_string_contains("nbRounds=9"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"id":"abc"}"#))
        .mount(&server)
        .await;
    let swiss = client(&server)
        .swiss()
        .edit("abc", 300, 0, 9)
        .send()
        .await
        .unwrap();
    assert_eq!(swiss.id, "abc");
}
