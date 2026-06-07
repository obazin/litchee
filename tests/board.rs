//! Integration tests for the Board API.

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
async fn stream_game_yields_full_then_state() {
    let server = MockServer::start().await;
    let body = concat!(
        r#"{"type":"gameFull","id":"g","white":{"id":"a","name":"A"},"black":{"id":"b","name":"B"},"state":{"type":"gameState","moves":"","wtime":1,"btime":1,"winc":0,"binc":0,"status":"started"}}"#,
        "\n",
        r#"{"type":"gameState","moves":"e2e4","wtime":1,"btime":1,"winc":0,"binc":0,"status":"started"}"#,
        "\n",
    );
    Mock::given(method("GET"))
        .and(path("/api/board/game/stream/g"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let stream = client(&server).board().stream_game("g").await.unwrap();
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
async fn make_move_sends_offering_draw_flag() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/board/game/g/move/e2e4"))
        .and(query_param("offeringDraw", "false"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;

    client(&server)
        .board()
        .make_move("g", "e2e4", false)
        .await
        .unwrap();
}

#[tokio::test]
async fn resign_posts_to_the_action_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/board/game/g/resign"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;

    client(&server).board().resign("g").await.unwrap();
}

#[tokio::test]
async fn handle_draw_uses_yes_no_segment() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/board/game/g/draw/yes"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;

    client(&server)
        .board()
        .handle_draw("g", true)
        .await
        .unwrap();
}

#[tokio::test]
async fn write_chat_posts_room_and_text() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/board/game/g/chat"))
        .and(body_string_contains("room=player"))
        .and(body_string_contains("text=hi"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;

    client(&server)
        .board()
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
    let stream = client(&server).board().stream_events().await.unwrap();
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

#[tokio::test]
async fn read_chat_returns_messages() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/board/game/g/chat"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"[{"user":"a","text":"gg"}]"#))
        .mount(&server)
        .await;
    let chat = client(&server).board().read_chat("g").await.unwrap();
    assert_eq!(chat[0].text, "gg");
}

#[tokio::test]
async fn seek_streams_keepalive() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/board/seek"))
        .and(body_string_contains("time=5"))
        .respond_with(ResponseTemplate::new(200).set_body_string("\n\n"))
        .mount(&server)
        .await;
    let stream = client(&server)
        .board()
        .seek()
        .rated(false)
        .clock(5.0, 3)
        .send()
        .await
        .unwrap();
    let items: Vec<_> = stream.collect().await;
    assert!(items.is_empty()); // keep-alive blank lines yield nothing
}
