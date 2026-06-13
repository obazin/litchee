//! Integration tests for the TV API.

use futures_util::StreamExt;
use litchee::LichessClient;
use litchee::api::broadcasting::tv::LichessTvFeedEvent;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> LichessClient {
    LichessClient::builder()
        .base_url(&server.uri().parse().expect("mock uri is a valid url"))
        .build()
        .expect("client builds")
}

#[tokio::test]
async fn channels_returns_featured_games() {
    let server = MockServer::start().await;
    let body = r#"{"bullet":{"user":{"id":"a","name":"A"},"rating":2900,
                   "gameId":"x","color":"white"}}"#;
    Mock::given(method("GET"))
        .and(path("/api/tv/channels"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let channels = client(&server).tv().channels().await.unwrap();

    assert_eq!(channels.bullet.unwrap().rating, 2900);
}

#[tokio::test]
async fn feed_streams_tagged_events() {
    let server = MockServer::start().await;
    let body = concat!(
        r#"{"t":"featured","d":{"id":"g","orientation":"white","players":[{"color":"white","rating":1500,"seconds":60},{"color":"black","rating":1490,"seconds":60}],"fen":"startpos"}}"#,
        "\n",
        r#"{"t":"fen","d":{"fen":"x","lm":"e2e4","wc":60,"bc":59}}"#,
        "\n",
    );
    Mock::given(method("GET"))
        .and(path("/api/tv/feed"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let stream = client(&server).tv().feed().await.unwrap();
    let events: Vec<_> = stream.collect().await;

    assert_eq!(events.len(), 2);
    assert!(matches!(
        events[0].as_ref().unwrap(),
        LichessTvFeedEvent::Featured(_)
    ));
    assert!(matches!(
        events[1].as_ref().unwrap(),
        LichessTvFeedEvent::Fen(_)
    ));
}

#[tokio::test]
async fn channel_feed_streams_tagged_events() {
    let server = MockServer::start().await;
    let body = concat!(
        r#"{"t":"featured","d":{"id":"g","orientation":"white","players":[{"color":"white","rating":1500,"seconds":60},{"color":"black","rating":1490,"seconds":60}],"fen":"startpos"}}"#,
        "\n",
        r#"{"t":"fen","d":{"fen":"x","lm":"e2e4","wc":60,"bc":59}}"#,
        "\n",
    );
    Mock::given(method("GET"))
        .and(path("/api/tv/blitz/feed"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let stream = client(&server).tv().channel_feed("blitz").await.unwrap();
    let events: Vec<_> = stream.collect().await;

    assert_eq!(events.len(), 2);
    assert!(matches!(
        events[0].as_ref().unwrap(),
        LichessTvFeedEvent::Featured(_)
    ));
}

#[tokio::test]
async fn channel_games_streams_games() {
    let server = MockServer::start().await;
    let body = "{\"id\":\"g1\"}\n{\"id\":\"g2\"}\n";
    Mock::given(method("GET"))
        .and(path("/api/tv/blitz"))
        .and(query_param("nb", "2"))
        .and(header("accept", "application/x-ndjson"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let stream = client(&server)
        .tv()
        .channel_games("blitz", 2)
        .await
        .unwrap();
    let games: Vec<_> = stream.collect().await;

    assert_eq!(games.len(), 2);
}
