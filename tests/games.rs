//! Integration tests for the Games API.

use futures_util::StreamExt;
use litchee::LichessClient;
use litchee::model::LichessColor;
use wiremock::matchers::{body_string_contains, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> LichessClient {
    LichessClient::builder()
        .base_url(&server.uri().parse().expect("mock uri is a valid url"))
        .token("test-token")
        .build()
        .expect("client builds")
}

#[tokio::test]
async fn export_single_game_as_json() {
    let server = MockServer::start().await;
    let body = r#"{"id":"q7ZvsdUF","rated":true,"winner":"white","status":"mate"}"#;
    Mock::given(method("GET"))
        .and(path("/game/export/q7ZvsdUF"))
        .and(query_param("evals", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let game = client(&server)
        .games()
        .export("q7ZvsdUF")
        .evals(true)
        .json()
        .await
        .unwrap();

    assert_eq!(game.id, "q7ZvsdUF");
    assert_eq!(game.winner, Some(LichessColor::White));
}

#[tokio::test]
async fn export_single_game_as_pgn_sets_accept_header() {
    let server = MockServer::start().await;
    let pgn = "[Event \"Casual\"]\n\n1. e4 e5 *";
    Mock::given(method("GET"))
        .and(path("/game/export/abcd1234"))
        .and(header("accept", "application/x-chess-pgn"))
        .respond_with(ResponseTemplate::new(200).set_body_string(pgn))
        .mount(&server)
        .await;

    let exported = client(&server)
        .games()
        .export("abcd1234")
        .pgn()
        .await
        .unwrap();

    assert!(exported.contains("1. e4 e5"));
}

#[tokio::test]
async fn export_user_games_streams_ndjson() {
    let server = MockServer::start().await;
    let body = "{\"id\":\"g1\"}\n{\"id\":\"g2\"}\n";
    Mock::given(method("GET"))
        .and(path("/api/games/user/bobby"))
        .and(query_param("max", "2"))
        .and(header("accept", "application/x-ndjson"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let stream = client(&server)
        .games()
        .export_user("bobby")
        .max(2)
        .rated(true)
        .stream()
        .await
        .unwrap();
    let games: Vec<_> = stream.collect().await;

    assert_eq!(games.len(), 2);
    assert_eq!(games[1].as_ref().unwrap().id, "g2");
}

#[tokio::test]
async fn import_game_posts_pgn() {
    let server = MockServer::start().await;
    let body = r#"{"id":"abcd1234","url":"https://lichess.org/abcd1234"}"#;
    Mock::given(method("POST"))
        .and(path("/api/import"))
        .and(body_string_contains("pgn="))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let imported = client(&server)
        .games()
        .import_game("1. e4 e5 *")
        .await
        .unwrap();

    assert_eq!(imported.id, "abcd1234");
}

#[tokio::test]
async fn now_playing_returns_games() {
    let server = MockServer::start().await;
    let body = r#"{"nowPlaying":[{"gameId":"g","fullId":"gf","color":"white","fen":"x",
        "opponent":{"id":"o","username":"O","rating":1500}}]}"#;
    Mock::given(method("GET"))
        .and(path("/api/account/playing"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;
    let playing = client(&server).games().now_playing().await.unwrap();
    assert_eq!(playing.now_playing[0].game_id, "g");
    assert_eq!(playing.now_playing[0].opponent.username, "O");
}

#[tokio::test]
async fn chat_returns_messages() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/game/g/chat"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(r#"[{"user":"Toby","text":"hi"}]"#),
        )
        .mount(&server)
        .await;
    let chat = client(&server).games().chat("g").await.unwrap();
    assert_eq!(chat[0].user, "Toby");
}

#[tokio::test]
async fn export_by_ids_streams_games() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/games/export/_ids"))
        .and(body_string_contains("a,b"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string("{\"id\":\"a\"}\n{\"id\":\"b\"}\n"),
        )
        .mount(&server)
        .await;
    let stream = client(&server)
        .games()
        .export_by_ids(&["a", "b"])
        .await
        .unwrap();
    let games: Vec<_> = stream.collect().await;
    assert_eq!(games.len(), 2);
}

#[tokio::test]
async fn stream_moves_yields_updates() {
    let server = MockServer::start().await;
    let body = "{\"fen\":\"x\",\"lastMove\":\"e2e4\"}\n{\"fen\":\"y\",\"lm\":\"e7e5\",\"wc\":100,\"bc\":99}\n";
    Mock::given(method("GET"))
        .and(path("/api/stream/game/g"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;
    let stream = client(&server).games().stream_moves("g").await.unwrap();
    let updates: Vec<_> = stream.collect().await;
    assert_eq!(updates.len(), 2);
    assert_eq!(updates[1].as_ref().unwrap().lm.as_deref(), Some("e7e5"));
}

#[tokio::test]
async fn current_game_returns_game() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/user/bobby/current-game"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"id":"cur123"}"#))
        .mount(&server)
        .await;
    let game = client(&server).games().current_game("bobby").await.unwrap();
    assert_eq!(game.id, "cur123");
}

#[tokio::test]
async fn export_user_games_as_pgn_sets_accept_header() {
    let server = MockServer::start().await;
    let pgn = "[Event \"Rated\"]\n\n1. d4 d5 *";
    Mock::given(method("GET"))
        .and(path("/api/games/user/bobby"))
        .and(header("accept", "application/x-chess-pgn"))
        .respond_with(ResponseTemplate::new(200).set_body_string(pgn))
        .mount(&server)
        .await;
    let exported = client(&server)
        .games()
        .export_user("bobby")
        .pgn()
        .await
        .unwrap();
    assert!(exported.contains("1. d4 d5"));
}

#[tokio::test]
async fn export_bookmarks_streams_games() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/games/export/bookmarks"))
        .and(header("accept", "application/x-ndjson"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string("{\"id\":\"b1\"}\n{\"id\":\"b2\"}\n"),
        )
        .mount(&server)
        .await;
    let stream = client(&server).games().export_bookmarks().await.unwrap();
    let games: Vec<_> = stream.collect().await;
    assert_eq!(games.len(), 2);
    assert_eq!(games[0].as_ref().unwrap().id, "b1");
}

#[tokio::test]
async fn export_imports_streams_games() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/games/export/imports"))
        .and(header("accept", "application/x-ndjson"))
        .respond_with(ResponseTemplate::new(200).set_body_string("{\"id\":\"i1\"}\n"))
        .mount(&server)
        .await;
    let stream = client(&server).games().export_imports().await.unwrap();
    let games: Vec<_> = stream.collect().await;
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].as_ref().unwrap().id, "i1");
}

#[tokio::test]
async fn stream_by_users_streams_games() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/stream/games-by-users"))
        .and(body_string_contains("alice,bob"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string("{\"id\":\"u1\"}\n{\"id\":\"u2\"}\n"),
        )
        .mount(&server)
        .await;
    let stream = client(&server)
        .games()
        .stream_by_users(&["alice", "bob"])
        .await
        .unwrap();
    let games: Vec<_> = stream.collect().await;
    assert_eq!(games.len(), 2);
}

#[tokio::test]
async fn stream_by_ids_streams_games() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/stream/games/mystream"))
        .and(body_string_contains("a,b"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string("{\"id\":\"a\"}\n{\"id\":\"b\"}\n"),
        )
        .mount(&server)
        .await;
    let stream = client(&server)
        .games()
        .stream_by_ids("mystream", &["a", "b"])
        .await
        .unwrap();
    let games: Vec<_> = stream.collect().await;
    assert_eq!(games.len(), 2);
}

#[tokio::test]
async fn add_to_stream_posts_ids() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/stream/games/mystream/add"))
        .and(body_string_contains("a,b"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;
    client(&server)
        .games()
        .add_to_stream("mystream", &["a", "b"])
        .await
        .unwrap();
}
