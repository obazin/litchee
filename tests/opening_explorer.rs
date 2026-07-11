//! Integration tests for the Opening Explorer API (explorer host routing).

use futures_util::StreamExt;
use litchee::LichessClient;
use litchee::api::database::opening_explorer::{ExplorerMode, RatingGroup};
use litchee::model::{LichessColor, LichessSpeed, LichessVariantKey};
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
        .masters("startpos")
        .send()
        .await
        .unwrap();

    assert_eq!(result.white, 100);
    assert_eq!(result.moves[0].san, "e4");
}

#[tokio::test]
async fn masters_sends_year_bounds_and_counts() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/masters"))
        .and(query_param("since", "1952"))
        .and(query_param("until", "2020"))
        .and(query_param("moves", "10"))
        .and(query_param("topGames", "15"))
        .respond_with(ResponseTemplate::new(200).set_body_string(RESULT))
        .mount(&server)
        .await;

    let result = client(&server)
        .opening_explorer()
        .masters("startpos")
        .since(1952)
        .until(2020)
        .moves(10)
        .top_games(15)
        .send()
        .await
        .unwrap();

    assert_eq!(result.white, 100);
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
        .lichess("startpos")
        .play("e2e4,e7e5")
        .send()
        .await
        .unwrap();

    assert_eq!(result.black, 60);
}

#[tokio::test]
async fn lichess_sends_speeds_and_ratings() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/lichess"))
        .and(query_param("speeds", "blitz,rapid"))
        .and(query_param("ratings", "1600,1800"))
        .respond_with(ResponseTemplate::new(200).set_body_string(RESULT))
        .mount(&server)
        .await;

    let result = client(&server)
        .opening_explorer()
        .lichess("startpos")
        .speeds(&[LichessSpeed::Blitz, LichessSpeed::Rapid])
        .ratings(&[RatingGroup::R1600, RatingGroup::R1800])
        .send()
        .await
        .unwrap();

    assert_eq!(result.black, 60);
}

#[tokio::test]
async fn lichess_sends_variant_dates_and_history() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/lichess"))
        .and(query_param("variant", "crazyhouse"))
        .and(query_param("since", "2015-01"))
        .and(query_param("until", "2020-12"))
        .and(query_param("topGames", "4"))
        .and(query_param("recentGames", "4"))
        .and(query_param("history", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_string(RESULT))
        .mount(&server)
        .await;

    let result = client(&server)
        .opening_explorer()
        .lichess("startpos")
        .variant(LichessVariantKey::Crazyhouse)
        .since("2015-01")
        .until("2020-12")
        .top_games(4)
        .recent_games(4)
        .history(true)
        .send()
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
        .player("bobby", LichessColor::White, "startpos")
        .stream()
        .await
        .unwrap();
    let results: Vec<_> = stream.collect().await;
    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn player_sends_speeds_modes_and_recent_games() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/player"))
        .and(query_param("player", "bobby"))
        .and(query_param("color", "black"))
        .and(query_param("variant", "standard"))
        .and(query_param("speeds", "bullet,blitz"))
        .and(query_param("modes", "rated"))
        .and(query_param("recentGames", "8"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string("{\"white\":1,\"draws\":0,\"black\":0,\"moves\":[]}\n"),
        )
        .mount(&server)
        .await;

    let stream = client(&server)
        .opening_explorer()
        .player("bobby", LichessColor::Black, "startpos")
        .variant(LichessVariantKey::Standard)
        .speeds(&[LichessSpeed::Bullet, LichessSpeed::Blitz])
        .modes(&[ExplorerMode::Rated])
        .recent_games(8)
        .stream()
        .await
        .unwrap();

    let results: Vec<_> = stream.collect().await;
    assert_eq!(results.len(), 1);
}
