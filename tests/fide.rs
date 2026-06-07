//! Integration tests for the FIDE API.

use litchee::LichessClient;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> LichessClient {
    LichessClient::builder()
        .base_url(&server.uri().parse().expect("mock uri is a valid url"))
        .build()
        .expect("client builds")
}

#[tokio::test]
async fn get_returns_a_player() {
    let server = MockServer::start().await;
    let body = r#"{"id":35009192,"name":"Erigaisi Arjun","federation":"IND","standard":2782}"#;
    Mock::given(method("GET"))
        .and(path("/api/fide/player/35009192"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let player = client(&server).fide().get(35_009_192).await.unwrap();

    assert_eq!(player.name, "Erigaisi Arjun");
    assert_eq!(player.standard, Some(2782));
}

#[tokio::test]
async fn ratings_returns_encoded_histories() {
    let server = MockServer::start().await;
    let body = r#"{"standard":[2015021577],"rapid":[],"blitz":[]}"#;
    Mock::given(method("GET"))
        .and(path("/api/fide/player/1/ratings"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let ratings = client(&server).fide().ratings(1).await.unwrap();

    assert_eq!(ratings.standard, vec![2_015_021_577]);
}

#[tokio::test]
async fn search_queries_the_term() {
    let server = MockServer::start().await;
    let body = r#"[{"id":1,"name":"A","federation":"FRA"}]"#;
    Mock::given(method("GET"))
        .and(path("/api/fide/player"))
        .and(query_param("q", "arjun"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let players = client(&server).fide().search("arjun").await.unwrap();

    assert_eq!(players.len(), 1);
}
