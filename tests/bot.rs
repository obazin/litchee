//! Integration tests for the Bot API.

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
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let stream = client(&server).bot().online().await.unwrap();
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
