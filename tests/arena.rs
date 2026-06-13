//! Integration tests for the Arena Tournaments API.

use futures_util::StreamExt;
use litchee::LichessClient;
use wiremock::matchers::{body_string_contains, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> LichessClient {
    LichessClient::builder()
        .base_url(&server.uri().parse().expect("mock uri is a valid url"))
        .token("test-token")
        .build()
        .expect("client builds")
}

#[tokio::test]
async fn list_returns_grouped_tournaments() {
    let server = MockServer::start().await;
    let body = r#"{"created":[],"started":[{"id":"abc","fullName":"Hourly","nbPlayers":50}],
        "finished":[]}"#;
    Mock::given(method("GET"))
        .and(path("/api/tournament"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let list = client(&server).arena().list().await.unwrap();

    assert_eq!(list.started[0].id, "abc");
}

#[tokio::test]
async fn create_posts_clock_and_name() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/tournament"))
        .and(body_string_contains("name=Blitz"))
        .and(body_string_contains("clockTime=3"))
        .and(body_string_contains("clockIncrement=0"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"id":"new1"}"#))
        .mount(&server)
        .await;

    let arena = client(&server)
        .arena()
        .create("Blitz", 3.0, 0, 60)
        .rated(true)
        .send()
        .await
        .unwrap();

    assert_eq!(arena.id, "new1");
}

#[tokio::test]
async fn join_posts_to_the_join_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/tournament/abc/join"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;

    client(&server).arena().join("abc").await.unwrap();
}

#[tokio::test]
async fn results_streams_rows() {
    let server = MockServer::start().await;
    let body = concat!(
        r#"{"rank":1,"score":20,"username":"A","rating":2400}"#,
        "\n",
        r#"{"rank":2,"score":18,"username":"B","rating":2350}"#,
        "\n",
    );
    Mock::given(method("GET"))
        .and(path("/api/tournament/abc/results"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let stream = client(&server).arena().results("abc").await.unwrap();
    let results: Vec<_> = stream.collect().await;

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].as_ref().unwrap().username, "A");
}

#[tokio::test]
async fn games_streams_with_ndjson_accept() {
    let server = MockServer::start().await;
    let body = "{\"id\":\"g1\"}\n{\"id\":\"g2\"}\n";
    Mock::given(method("GET"))
        .and(path("/api/tournament/abc/games"))
        .and(header("accept", "application/x-ndjson"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let stream = client(&server).arena().games("abc").await.unwrap();
    let games: Vec<_> = stream.collect().await;

    assert_eq!(games.len(), 2);
}

#[tokio::test]
async fn update_posts_to_id_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/tournament/abc"))
        .and(body_string_contains("name=Renamed"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"id":"abc"}"#))
        .mount(&server)
        .await;
    let arena = client(&server)
        .arena()
        .update("abc", "Renamed", 3.0, 0, 60)
        .send()
        .await
        .unwrap();
    assert_eq!(arena.id, "abc");
}

#[tokio::test]
async fn teams_returns_standings() {
    let server = MockServer::start().await;
    let body = r#"{"id":"abc","teams":[{"rank":1,"id":"t1","score":100}]}"#;
    Mock::given(method("GET"))
        .and(path("/api/tournament/abc/teams"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;
    let standings = client(&server).arena().teams("abc").await.unwrap();
    assert_eq!(standings.teams[0].id, "t1");
}

#[tokio::test]
async fn created_by_streams_arenas() {
    let server = MockServer::start().await;
    let body = "{\"id\":\"a\"}\n{\"id\":\"b\"}\n";
    Mock::given(method("GET"))
        .and(path("/api/user/bob/tournament/created"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;
    let stream = client(&server).arena().created_by("bob").await.unwrap();
    let arenas: Vec<_> = stream.collect().await;
    assert_eq!(arenas.len(), 2);
}

#[tokio::test]
async fn get_returns_full_arena() {
    let server = MockServer::start().await;
    let body = r#"{"id":"abc","fullName":"Hourly","nbPlayers":50}"#;
    Mock::given(method("GET"))
        .and(path("/api/tournament/abc"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;
    let arena = client(&server).arena().get("abc").await.unwrap();
    assert_eq!(arena.id, "abc");
}

#[tokio::test]
async fn setup_team_battle_posts_teams() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/tournament/team-battle/abc"))
        .and(body_string_contains("teams=t1%2Ct2"))
        .and(body_string_contains("nbLeaders=3"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"id":"abc"}"#))
        .mount(&server)
        .await;
    let arena = client(&server)
        .arena()
        .setup_team_battle("abc", &["t1", "t2"], 3)
        .await
        .unwrap();
    assert_eq!(arena.id, "abc");
}

#[tokio::test]
async fn played_by_streams_arenas() {
    let server = MockServer::start().await;
    let body = "{\"id\":\"a\"}\n{\"id\":\"b\"}\n";
    Mock::given(method("GET"))
        .and(path("/api/user/bob/tournament/played"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;
    let stream = client(&server).arena().played_by("bob").await.unwrap();
    let arenas: Vec<_> = stream.collect().await;
    assert_eq!(arenas.len(), 2);
}

#[tokio::test]
async fn withdraw_posts_to_the_withdraw_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/tournament/abc/withdraw"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;
    client(&server).arena().withdraw("abc").await.unwrap();
}

#[tokio::test]
async fn terminate_posts_to_the_terminate_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/tournament/abc/terminate"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;
    client(&server).arena().terminate("abc").await.unwrap();
}
