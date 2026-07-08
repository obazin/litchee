//! Integration tests for the Broadcasts API.

use futures_util::StreamExt;
use litchee::LichessClient;
use litchee::api::broadcasting::broadcasts::BroadcastTourInfo;
use litchee::model::PgnExportOptions;
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
        .and(query_param("nb", "20"))
        .and(query_param("live", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let stream = client(&server)
        .broadcasts()
        .official(Some(20), None, Some(true))
        .await
        .unwrap();
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
        .and(query_param("clocks", "false"))
        .and(query_param("comments", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_string("[Event \"Round 1\"]\n\n1. e4 *"))
        .mount(&server)
        .await;

    let pgn = client(&server)
        .broadcasts()
        .round_pgn(
            "r1",
            &PgnExportOptions::default().clocks(false).comments(true),
        )
        .await
        .unwrap();

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
        .and(query_param("page", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;
    let top = client(&server)
        .broadcasts()
        .top(Some(1), None)
        .await
        .unwrap();
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
        .and(body_string_contains("tier=5"))
        .and(body_string_contains("showScores=true"))
        .and(body_string_contains("info.format=8-player+round-robin"))
        .and(body_string_contains("info.tc=Classical"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;
    let tour = client(&server)
        .broadcasts()
        .create_tour("My Tour")
        .tier(5)
        .show_scores(true)
        .info(
            BroadcastTourInfo::default()
                .format("8-player round-robin")
                .tc("Classical"),
        )
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

#[tokio::test]
async fn by_user_streams_broadcasts() {
    let server = MockServer::start().await;
    let body = concat!(
        r#"{"tour":{"id":"a","name":"One"},"rounds":[]}"#,
        "\n",
        r#"{"tour":{"id":"b","name":"Two"},"rounds":[]}"#,
        "\n",
    );
    Mock::given(method("GET"))
        .and(path("/api/broadcast/by/thibault"))
        .and(query_param("page", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let stream = client(&server)
        .broadcasts()
        .by_user("thibault", Some(2), None)
        .await
        .unwrap();
    let broadcasts: Vec<_> = stream.collect().await;

    assert_eq!(broadcasts.len(), 2);
    assert_eq!(broadcasts[1].as_ref().unwrap().tour.id, "b");
}

#[tokio::test]
async fn my_rounds_streams_rounds() {
    let server = MockServer::start().await;
    let body = concat!(
        r#"{"round":{"id":"r1","name":"R1"}}"#,
        "\n",
        r#"{"round":{"id":"r2","name":"R2"}}"#,
        "\n",
    );
    Mock::given(method("GET"))
        .and(path("/api/broadcast/my-rounds"))
        .and(query_param("nb", "5"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let stream = client(&server)
        .broadcasts()
        .my_rounds(Some(5))
        .await
        .unwrap();
    let rounds: Vec<_> = stream.collect().await;

    assert_eq!(rounds.len(), 2);
    assert!(rounds[0].as_ref().unwrap().data.contains_key("round"));
}

#[tokio::test]
async fn round_returns_round_view() {
    let server = MockServer::start().await;
    let body = r#"{"tour":{"id":"abc","name":"World Champ"},"round":{"id":"r1"},"games":[]}"#;
    Mock::given(method("GET"))
        .and(path("/api/broadcast/wc/round-1/r1"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let view = client(&server)
        .broadcasts()
        .round("wc", "round-1", "r1")
        .await
        .unwrap();

    assert_eq!(view.tour.unwrap().id, "abc");
}

#[tokio::test]
async fn all_rounds_pgn_returns_text() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/broadcast/abc.pgn"))
        .respond_with(ResponseTemplate::new(200).set_body_string("[Event \"Tour\"]\n\n1. e4 *"))
        .mount(&server)
        .await;

    let pgn = client(&server)
        .broadcasts()
        .all_rounds_pgn("abc", &PgnExportOptions::default())
        .await
        .unwrap();

    assert!(pgn.contains("Tour"));
}

#[tokio::test]
async fn stream_round_pgn_returns_text() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/stream/broadcast/round/r1.pgn"))
        .respond_with(ResponseTemplate::new(200).set_body_string("[Event \"Live\"]\n\n1. d4 *"))
        .mount(&server)
        .await;

    let pgn = client(&server)
        .broadcasts()
        .stream_round_pgn("r1", &PgnExportOptions::default())
        .await
        .unwrap();

    assert!(pgn.contains("Live"));
}

#[tokio::test]
async fn stream_group_pgn_returns_text() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/stream/broadcast/group/albQx5zq.pgn"))
        .respond_with(ResponseTemplate::new(200).set_body_string("[Event \"Group\"]\n\n1. c4 *"))
        .mount(&server)
        .await;

    let pgn = client(&server)
        .broadcasts()
        .stream_group_pgn("albQx5zq", &PgnExportOptions::default())
        .await
        .unwrap();

    assert!(pgn.contains("Group"));
}

#[tokio::test]
async fn player_returns_a_single_entry() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/broadcast/t1/players/p1"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(r#"{"name":"Carlsen","fideId":1503014}"#),
        )
        .mount(&server)
        .await;

    let player = client(&server)
        .broadcasts()
        .player("t1", "p1")
        .await
        .unwrap();

    assert_eq!(player.name.as_deref(), Some("Carlsen"));
}

#[tokio::test]
async fn team_standings_returns_entries() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/broadcast/t1/teams/standings"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"[{"name":"Team A"}]"#))
        .mount(&server)
        .await;

    let standings = client(&server)
        .broadcasts()
        .team_standings("t1")
        .await
        .unwrap();

    assert_eq!(standings[0].name.as_deref(), Some("Team A"));
}

#[tokio::test]
async fn update_tour_posts_to_edit() {
    let server = MockServer::start().await;
    let body = r#"{"tour":{"id":"t1","name":"Renamed"},"rounds":[]}"#;
    Mock::given(method("POST"))
        .and(path("/broadcast/t1/edit"))
        .and(body_string_contains("name=Renamed"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let tour = client(&server)
        .broadcasts()
        .update_tour("t1", "Renamed")
        .send()
        .await
        .unwrap();

    assert_eq!(tour.tour.name, "Renamed");
}

#[tokio::test]
async fn create_round_posts_to_new() {
    let server = MockServer::start().await;
    let body = r#"{"round":{"id":"r1"},"games":[]}"#;
    Mock::given(method("POST"))
        .and(path("/broadcast/t1/new"))
        .and(body_string_contains("name=Round+1"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let view = client(&server)
        .broadcasts()
        .create_round("t1", "Round 1")
        .send()
        .await
        .unwrap();

    assert!(view.round.is_some());
}

#[tokio::test]
async fn create_round_posts_sync_and_scoring_fields() {
    let server = MockServer::start().await;
    let body = r#"{"round":{"id":"r1"},"games":[]}"#;
    Mock::given(method("POST"))
        .and(path("/broadcast/t1/new"))
        .and(body_string_contains("syncUrl=https"))
        .and(body_string_contains("period=30"))
        .and(body_string_contains("rated=true"))
        .and(body_string_contains("onlyRound=3"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let view = client(&server)
        .broadcasts()
        .create_round("t1", "Round 1")
        .sync_url("https://example.org/games.pgn")
        .period(30)
        .rated(true)
        .only_round(3)
        .send()
        .await
        .unwrap();

    assert!(view.round.is_some());
}

#[tokio::test]
async fn update_round_sends_patch_query() {
    let server = MockServer::start().await;
    let body = r#"{"round":{"id":"r1"},"games":[]}"#;
    Mock::given(method("POST"))
        .and(path("/broadcast/round/r1/edit"))
        .and(query_param("patch", "true"))
        .and(body_string_contains("delay=60"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let view = client(&server)
        .broadcasts()
        .update_round("r1", "Round 1")
        .delay(60)
        .patch(true)
        .send()
        .await
        .unwrap();

    assert!(view.round.is_some());
}

#[tokio::test]
async fn update_round_posts_to_edit() {
    let server = MockServer::start().await;
    let body = r#"{"round":{"id":"r1"},"games":[]}"#;
    Mock::given(method("POST"))
        .and(path("/broadcast/round/r1/edit"))
        .and(body_string_contains("name=Renamed+Round"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let view = client(&server)
        .broadcasts()
        .update_round("r1", "Renamed Round")
        .send()
        .await
        .unwrap();

    assert!(view.round.is_some());
}
