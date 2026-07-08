//! Integration tests for the Bot API.

use futures_util::StreamExt;
use litchee::LichessClient;
use litchee::api::gameplay::board::{LichessBoardEvent, LichessChatRoom, LichessIncomingEvent};
use wiremock::matchers::{body_string_contains, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> LichessClient {
    LichessClient::builder()
        .base_url(&server.uri().parse().expect("mock uri is a valid url"))
        .token("test-token")
        .build()
        .expect("client builds")
}

#[tokio::test]
async fn upgrade_to_bot_posts_to_upgrade() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/bot/account/upgrade"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;

    client(&server).bot().upgrade_to_bot().await.unwrap();
}

#[tokio::test]
async fn online_streams_bot_users() {
    let server = MockServer::start().await;
    let body = "{\"id\":\"bot1\",\"username\":\"Bot1\",\"title\":\"BOT\"}\n{\"id\":\"bot2\",\"username\":\"Bot2\"}\n";
    Mock::given(method("GET"))
        .and(path("/api/bot/online"))
        .and(query_param("nb", "50"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let stream = client(&server).bot().online(Some(50)).await.unwrap();
    let bots: Vec<_> = stream.collect().await;

    assert_eq!(bots.len(), 2);
    assert_eq!(bots[0].as_ref().unwrap().username, "Bot1");
}

#[tokio::test]
async fn make_move_posts_to_bot_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/bot/game/g/move/e2e4"))
        .and(query_param("offeringDraw", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;

    client(&server)
        .bot()
        .make_move("g", "e2e4", true)
        .await
        .unwrap();
}

#[tokio::test]
async fn handle_takeback_uses_no_segment() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/bot/game/g/takeback/no"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;

    client(&server)
        .bot()
        .handle_takeback("g", false)
        .await
        .unwrap();
}

#[tokio::test]
async fn read_chat_returns_messages() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/bot/game/g/chat"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"[{"user":"a","text":"gg"}]"#))
        .mount(&server)
        .await;
    let chat = client(&server).bot().read_chat("g").await.unwrap();
    assert_eq!(chat[0].text, "gg");
}

#[tokio::test]
async fn stream_game_yields_full_then_state() {
    let server = MockServer::start().await;
    let body = concat!(
        r#"{"type":"gameFull","id":"g","white":{"id":"a","name":"A"},"black":{"id":"b","name":"B"},"state":{"type":"gameState","moves":"","wtime":1,"btime":1,"winc":0,"binc":0,"status":"started"}}"#,
        "\n",
        r#"{"type":"gameState","moves":"e2e4","wtime":1,"btime":1,"winc":0,"binc":0,"status":"started"}"#,
        "\n",
    );
    Mock::given(method("GET"))
        .and(path("/api/bot/game/stream/g"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let stream = client(&server).bot().stream_game("g").await.unwrap();
    let events: Vec<_> = stream.collect().await;

    assert_eq!(events.len(), 2);
    assert!(matches!(
        events[0].as_ref().unwrap(),
        LichessBoardEvent::GameFull(_)
    ));
    assert!(matches!(
        events[1].as_ref().unwrap(),
        LichessBoardEvent::GameState(_)
    ));
}

#[tokio::test]
async fn abort_posts_to_the_action_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/bot/game/g/abort"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;

    client(&server).bot().abort("g").await.unwrap();
}

#[tokio::test]
async fn resign_posts_to_the_action_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/bot/game/g/resign"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;

    client(&server).bot().resign("g").await.unwrap();
}

#[tokio::test]
async fn claim_victory_posts_to_the_action_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/bot/game/g/claim-victory"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;

    client(&server).bot().claim_victory("g").await.unwrap();
}

#[tokio::test]
async fn claim_draw_posts_to_the_action_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/bot/game/g/claim-draw"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;

    client(&server).bot().claim_draw("g").await.unwrap();
}

#[tokio::test]
async fn handle_draw_uses_yes_no_segment() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/bot/game/g/draw/yes"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;

    client(&server).bot().handle_draw("g", true).await.unwrap();
}

#[tokio::test]
async fn write_chat_posts_room_and_text() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/bot/game/g/chat"))
        .and(body_string_contains("room=player"))
        .and(body_string_contains("text=hi"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;

    client(&server)
        .bot()
        .write_chat("g", LichessChatRoom::Player, "hi")
        .await
        .unwrap();
}

#[tokio::test]
async fn stream_events_yields_incoming_events() {
    let server = MockServer::start().await;
    let body = concat!(
        r#"{"type":"gameStart","game":{"gameId":"g","color":"white","fen":"x"}}"#,
        "\n",
        r#"{"type":"challenge","challenge":{"id":"c","url":"u","status":"created"}}"#,
        "\n",
    );
    Mock::given(method("GET"))
        .and(path("/api/stream/event"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;
    let stream = client(&server).bot().stream_events().await.unwrap();
    let events: Vec<_> = stream.collect().await;
    assert_eq!(events.len(), 2);
    assert!(matches!(
        events[0].as_ref().unwrap(),
        LichessIncomingEvent::GameStart { .. }
    ));
    assert!(matches!(
        events[1].as_ref().unwrap(),
        LichessIncomingEvent::Challenge { .. }
    ));
}
