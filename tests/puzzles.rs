//! Integration tests for the Puzzles API.

use futures_util::StreamExt;
use litchee::LichessClient;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> LichessClient {
    LichessClient::builder()
        .base_url(&server.uri().parse().expect("mock uri is a valid url"))
        .token("test-token")
        .build()
        .expect("client builds")
}

const PUZZLE: &str = r#"{"game":{"id":"g",
    "players":[{"color":"white","id":"a","name":"A","rating":1500},
               {"color":"black","id":"b","name":"B","rating":1490}]},
    "puzzle":{"id":"p","initialPly":20,"plays":100,"rating":1600,
              "solution":["e2e4"],"themes":["mateIn1"]}}"#;

#[tokio::test]
async fn daily_returns_puzzle_and_game() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/puzzle/daily"))
        .respond_with(ResponseTemplate::new(200).set_body_string(PUZZLE))
        .mount(&server)
        .await;

    let pg = client(&server).puzzles().daily().await.unwrap();

    assert_eq!(pg.puzzle.id, "p");
}

#[tokio::test]
async fn get_fetches_a_puzzle_by_id() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/puzzle/p"))
        .respond_with(ResponseTemplate::new(200).set_body_string(PUZZLE))
        .mount(&server)
        .await;

    let pg = client(&server).puzzles().get("p").await.unwrap();

    assert_eq!(pg.puzzle.rating, Some(1600));
}

#[tokio::test]
async fn next_sends_angle_and_difficulty() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/puzzle/next"))
        .and(query_param("angle", "mateIn2"))
        .and(query_param("difficulty", "harder"))
        .respond_with(ResponseTemplate::new(200).set_body_string(PUZZLE))
        .mount(&server)
        .await;

    let pg = client(&server)
        .puzzles()
        .next(Some("mateIn2"), Some("harder"), None)
        .await
        .unwrap();

    assert_eq!(pg.puzzle.id, "p");
}

#[tokio::test]
async fn activity_streams_rounds() {
    let server = MockServer::start().await;
    let body = concat!(
        r#"{"date":1,"win":true,"puzzle":{"id":"p1","fen":"x","lastMove":"a1a2","plays":1,"rating":1500,"solution":["a1a2"],"themes":["t"]}}"#,
        "\n",
        r#"{"date":2,"win":false,"puzzle":{"id":"p2","fen":"y","lastMove":"b1b2","plays":2,"rating":1510,"solution":["b1b2"],"themes":["t"]}}"#,
        "\n",
    );
    Mock::given(method("GET"))
        .and(path("/api/puzzle/activity"))
        .and(query_param("max", "2"))
        .and(query_param("since", "1700000000000"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let stream = client(&server)
        .puzzles()
        .activity(Some(2), None, Some(1_700_000_000_000))
        .await
        .unwrap();
    let rounds: Vec<_> = stream.collect().await;

    assert_eq!(rounds.len(), 2);
    assert!(rounds[0].as_ref().unwrap().win);
}

#[tokio::test]
async fn batch_returns_puzzles() {
    let server = MockServer::start().await;
    let body = format!(r#"{{"puzzles":[{PUZZLE}]}}"#);
    Mock::given(method("GET"))
        .and(path("/api/puzzle/batch/mix"))
        .and(query_param("nb", "5"))
        .and(query_param("difficulty", "hardest"))
        .and(query_param("color", "white"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;
    let batch = client(&server)
        .puzzles()
        .batch("mix", 5, Some("hardest"), Some("white"))
        .await
        .unwrap();
    assert_eq!(batch.puzzles.len(), 1);
}

#[tokio::test]
async fn racer_get_and_create() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/racer/r1"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"id":"r1","url":"u"}"#))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/racer"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"id":"r2","url":"u2"}"#))
        .mount(&server)
        .await;
    assert_eq!(
        client(&server).puzzles().racer("r1").await.unwrap().id,
        "r1"
    );
    assert_eq!(
        client(&server).puzzles().create_racer().await.unwrap().id,
        "r2"
    );
}

#[tokio::test]
async fn dashboard_and_storm() {
    use litchee::api::training::puzzles::LichessPuzzleSolution;
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/puzzle/dashboard/30"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(r#"{"days":30,"global":{},"themes":{}}"#),
        )
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/puzzle/batch/mix"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"puzzles":[]}"#))
        .mount(&server)
        .await;
    let dash = client(&server).puzzles().dashboard(30).await.unwrap();
    assert_eq!(dash.days, Some(30));
    let sols = [LichessPuzzleSolution::new("p1", true)];
    let batch = client(&server)
        .puzzles()
        .solve_batch("mix", &sols, 0)
        .await
        .unwrap();
    assert!(batch.puzzles.is_empty());
}

#[tokio::test]
async fn solve_batch_sends_nb_and_decodes_rounds() {
    use litchee::api::training::puzzles::LichessPuzzleSolution;
    let server = MockServer::start().await;
    let body = format!(
        r#"{{"puzzles":[{PUZZLE}],
             "rounds":[{{"id":"p1","win":true,"ratingDiff":7}}]}}"#
    );
    Mock::given(method("POST"))
        .and(path("/api/puzzle/batch/mix"))
        .and(query_param("nb", "3"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let sols = [LichessPuzzleSolution::new("p1", true)];
    let resp = client(&server)
        .puzzles()
        .solve_batch("mix", &sols, 3)
        .await
        .unwrap();

    assert_eq!(resp.puzzles.len(), 1);
    assert_eq!(resp.rounds.len(), 1);
    assert_eq!(resp.rounds[0].rating_diff, Some(7));
}

#[tokio::test]
async fn replay_returns_session() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/puzzle/replay/30/mateIn1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"{"angle":{"key":"mateIn1"},"replay":{"days":30}}"#),
        )
        .mount(&server)
        .await;

    let replay = client(&server)
        .puzzles()
        .replay(30, "mateIn1")
        .await
        .unwrap();

    assert!(replay.replay.is_some());
}

#[tokio::test]
async fn storm_dashboard_returns_scores() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/storm/dashboard/maia"))
        .and(query_param("days", "30"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(r#"{"high":{"allTime":50},"days":[]}"#),
        )
        .mount(&server)
        .await;

    let dash = client(&server)
        .puzzles()
        .storm_dashboard("maia", Some(30))
        .await
        .unwrap();

    assert!(dash.high.is_some());
}
