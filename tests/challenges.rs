//! Integration tests for the Challenges API.

use futures_util::StreamExt;
use litchee::LichessClient;
use litchee::api::gameplay::challenges::LichessChallengeColor;
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
async fn list_returns_in_and_out() {
    let server = MockServer::start().await;
    let body = r#"{"in":[{"id":"a","url":"u","status":"created"}],"out":[]}"#;
    Mock::given(method("GET"))
        .and(path("/api/challenge"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let challenges = client(&server).challenges().list().await.unwrap();

    assert_eq!(challenges.incoming.len(), 1);
    assert!(challenges.outgoing.is_empty());
}

#[tokio::test]
async fn challenge_user_posts_clock_form_fields() {
    let server = MockServer::start().await;
    let body = r#"{"id":"H9fIRZUk","url":"u","status":"created"}"#;
    Mock::given(method("POST"))
        .and(path("/api/challenge/bobby"))
        .and(body_string_contains("clock.limit=600"))
        .and(body_string_contains("color=white"))
        .and(body_string_contains("rules=noRematch"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let challenge = client(&server)
        .challenges()
        .challenge("bobby")
        .rated(true)
        .clock(600, 0)
        .color(LichessChallengeColor::White)
        .rules("noRematch")
        .send()
        .await
        .unwrap();

    assert_eq!(challenge.id, "H9fIRZUk");
}

#[tokio::test]
async fn challenge_stream_sends_keep_alive_and_streams_status() {
    let server = MockServer::start().await;
    let body = "{\"challenge\":{\"id\":\"c1\"}}\n{\"done\":\"accepted\"}\n";
    Mock::given(method("POST"))
        .and(path("/api/challenge/bobby"))
        .and(body_string_contains("keepAliveStream=true"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let stream = client(&server)
        .challenges()
        .challenge("bobby")
        .clock(600, 0)
        .stream()
        .await
        .unwrap();
    let updates: Vec<_> = stream.collect().await;
    assert_eq!(updates.len(), 2);
    assert_eq!(updates[1].as_ref().unwrap()["done"], "accepted");
}

#[tokio::test]
async fn challenge_ai_returns_a_game() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/challenge/ai"))
        .and(body_string_contains("level=5"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"id":"aiGame12"}"#))
        .mount(&server)
        .await;

    let game = client(&server)
        .challenges()
        .challenge_ai(5)
        .clock(300, 3)
        .send()
        .await
        .unwrap();

    assert_eq!(game.id, "aiGame12");
}

#[tokio::test]
async fn accept_posts_to_accept_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/challenge/abc/accept"))
        .and(query_param("color", "white"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;

    client(&server)
        .challenges()
        .accept("abc", Some("white"))
        .await
        .unwrap();
}

#[tokio::test]
async fn decline_sends_reason() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/challenge/abc/decline"))
        .and(body_string_contains("reason=later"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;

    client(&server)
        .challenges()
        .decline("abc", Some("later"))
        .await
        .unwrap();
}

#[tokio::test]
async fn add_time_posts_to_round_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/round/g/add-time/15"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;

    client(&server)
        .challenges()
        .add_time("g", 15)
        .await
        .unwrap();
}

#[tokio::test]
async fn create_open_posts_form() {
    let server = MockServer::start().await;
    let body = r#"{"id":"o1","url":"u","status":"created"}"#;
    Mock::given(method("POST"))
        .and(path("/api/challenge/open"))
        .and(body_string_contains("clock.limit=300"))
        .and(body_string_contains("rules=noRematch"))
        .and(body_string_contains("users=alice%2Cbob"))
        .and(body_string_contains("expiresAt=1700000000000"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;
    let open = client(&server)
        .challenges()
        .create_open()
        .rated(false)
        .clock(300, 0)
        .rules("noRematch")
        .users("alice,bob")
        .expires_at(1_700_000_000_000)
        .send()
        .await
        .unwrap();
    assert_eq!(open.id, "o1");
}

#[tokio::test]
async fn start_clocks_sends_tokens() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/challenge/g/start-clocks"))
        .and(query_param("token1", "t1"))
        .and(query_param("token2", "t2"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;
    client(&server)
        .challenges()
        .start_clocks("g", "t1", "t2")
        .await
        .unwrap();
}

#[tokio::test]
async fn admin_challenge_tokens_returns_map() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/token/admin-challenge"))
        .and(body_string_contains("users=a%2Cb"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"a":"tok_a","b":"tok_b"}"#))
        .mount(&server)
        .await;
    let tokens = client(&server)
        .challenges()
        .admin_challenge_tokens(&["a", "b"], "test")
        .await
        .unwrap();
    assert_eq!(tokens["a"], "tok_a");
}

#[tokio::test]
async fn show_returns_challenge() {
    let server = MockServer::start().await;
    let body = r#"{"id":"H9fIRZUk","url":"u","status":"created"}"#;
    Mock::given(method("GET"))
        .and(path("/api/challenge/H9fIRZUk/show"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let challenge = client(&server).challenges().show("H9fIRZUk").await.unwrap();

    assert_eq!(challenge.id, "H9fIRZUk");
}

#[tokio::test]
async fn cancel_posts_to_cancel_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/challenge/abc/cancel"))
        .and(query_param("opponentToken", "tok"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;

    client(&server)
        .challenges()
        .cancel("abc", Some("tok"))
        .await
        .unwrap();
}
