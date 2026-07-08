//! Integration tests for the Swiss Tournaments API.

use futures_util::StreamExt;
use litchee::LichessClient;
use litchee::api::tournaments::swiss::SwissConditions;
use litchee::error::{ApiErrorKind, LichessError};
use litchee::model::GameExportOptions;
use wiremock::matchers::{
    body_string_contains, header, method, path, query_param, query_param_is_missing,
};
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
async fn create_posts_conditions_and_extra_fields() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/swiss/new/coders"))
        .and(body_string_contains("password=secret"))
        .and(body_string_contains("forbiddenPairings="))
        .and(body_string_contains("conditions.minRating.rating=1500"))
        .and(body_string_contains("conditions.playYourGames=true"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"id":"new2"}"#))
        .mount(&server)
        .await;

    let swiss = client(&server)
        .swiss()
        .create("coders", 300, 0, 7)
        .password("secret")
        .forbidden_pairings("alice bob")
        .conditions(
            SwissConditions::default()
                .min_rating(1500)
                .play_your_games(true),
        )
        .send()
        .await
        .unwrap();

    assert_eq!(swiss.id, "new2");
}

#[tokio::test]
async fn create_401_stays_generic_unauthorized() {
    // Only edit/schedule remap 401 to SwissUnauthorizedEdit; create must not.
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/swiss/new/coders"))
        .respond_with(ResponseTemplate::new(401).set_body_string(r#"{"error":"x"}"#))
        .mount(&server)
        .await;
    let err = client(&server)
        .swiss()
        .create("coders", 300, 0, 7)
        .send()
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        LichessError::Api(api) if api.kind == ApiErrorKind::Unauthorized
    ));
}

#[tokio::test]
async fn join_posts_to_the_join_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/swiss/abc/join"))
        .and(body_string_contains("password=secret"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;

    client(&server)
        .swiss()
        .join("abc", Some("secret"))
        .await
        .unwrap();
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
        .and(query_param("nb", "50"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let stream = client(&server)
        .swiss()
        .results("abc", Some(50))
        .await
        .unwrap();
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

#[tokio::test]
async fn edit_401_maps_to_swiss_unauthorized_edit() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/swiss/abc/edit"))
        .respond_with(
            ResponseTemplate::new(401)
                .set_body_string(r#"{"error":"This user cannot edit this swiss"}"#),
        )
        .mount(&server)
        .await;
    let err = client(&server)
        .swiss()
        .edit("abc", 300, 0, 9)
        .send()
        .await
        .unwrap_err();
    match err {
        LichessError::Api(api) => {
            assert_eq!(api.kind, ApiErrorKind::SwissUnauthorizedEdit);
            assert_eq!(api.status.as_u16(), 401);
            assert_eq!(
                api.message.as_deref(),
                Some("This user cannot edit this swiss")
            );
        }
        other => panic!("expected Api error, got {other:?}"),
    }
}

#[tokio::test]
async fn schedule_next_round_401_maps_to_swiss_unauthorized_edit() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/swiss/abc/schedule-next-round"))
        .respond_with(ResponseTemplate::new(401).set_body_string(r#"{"error":"nope"}"#))
        .mount(&server)
        .await;
    let err = client(&server)
        .swiss()
        .schedule_next_round("abc", None)
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        LichessError::Api(api) if api.kind == ApiErrorKind::SwissUnauthorizedEdit
    ));
}

#[tokio::test]
async fn withdraw_posts_to_the_withdraw_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/swiss/abc/withdraw"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;
    client(&server).swiss().withdraw("abc").await.unwrap();
}

#[tokio::test]
async fn terminate_posts_to_the_terminate_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/swiss/abc/terminate"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;
    client(&server).swiss().terminate("abc").await.unwrap();
}

#[tokio::test]
async fn schedule_next_round_posts_to_the_schedule_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/swiss/abc/schedule-next-round"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;
    client(&server)
        .swiss()
        .schedule_next_round("abc", None)
        .await
        .unwrap();
}

#[tokio::test]
async fn games_streams_with_ndjson_accept() {
    let server = MockServer::start().await;
    let body = "{\"id\":\"g1\"}\n{\"id\":\"g2\"}\n";
    Mock::given(method("GET"))
        .and(path("/api/swiss/abc/games"))
        .and(header("accept", "application/x-ndjson"))
        .and(query_param_is_missing("player"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;
    let stream = client(&server).swiss().games("abc").stream().await.unwrap();
    let games: Vec<_> = stream.collect().await;
    assert_eq!(games.len(), 2);
}

#[tokio::test]
async fn games_sends_player_and_export_params() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/swiss/abc/games"))
        .and(query_param("player", "bobby"))
        .and(query_param("clocks", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_string("{\"id\":\"g1\"}\n"))
        .mount(&server)
        .await;
    let stream = client(&server)
        .swiss()
        .games("abc")
        .player("bobby")
        .export(GameExportOptions::default().clocks(true))
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
        .and(path("/api/swiss/abc/games"))
        .and(header("accept", "application/x-chess-pgn"))
        .respond_with(ResponseTemplate::new(200).set_body_string("[Event \"x\"]\n\n1. e4 *"))
        .mount(&server)
        .await;
    let pgn = client(&server).swiss().games("abc").pgn().await.unwrap();
    assert!(pgn.contains("1. e4"));
}
