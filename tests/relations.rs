//! Integration tests for the Relations API.

use futures_util::StreamExt;
use litchee::LichessClient;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> LichessClient {
    LichessClient::builder()
        .base_url(&server.uri().parse().expect("mock uri is a valid url"))
        .token("test-token")
        .build()
        .expect("client builds")
}

#[tokio::test]
async fn following_streams_ndjson_users() {
    let server = MockServer::start().await;
    let body = "{\"id\":\"a\",\"username\":\"A\"}\n{\"id\":\"b\",\"username\":\"B\"}\n";
    Mock::given(method("GET"))
        .and(path("/api/rel/following"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let stream = client(&server).relations().following().await.unwrap();
    let users: Vec<_> = stream.collect().await;

    assert_eq!(users.len(), 2);
    assert_eq!(users[0].as_ref().unwrap().id, "a");
    assert_eq!(users[1].as_ref().unwrap().username, "B");
}

#[tokio::test]
async fn follow_posts_to_the_user_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/rel/follow/bobby"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;

    client(&server).relations().follow("bobby").await.unwrap();
}

#[tokio::test]
async fn unfollow_posts_to_the_user_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/rel/unfollow/bobby"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;

    client(&server).relations().unfollow("bobby").await.unwrap();
}

#[tokio::test]
async fn block_posts_to_the_user_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/rel/block/bobby"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;

    client(&server).relations().block("bobby").await.unwrap();
}

#[tokio::test]
async fn unblock_posts_to_the_user_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/rel/unblock/bobby"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;

    client(&server).relations().unblock("bobby").await.unwrap();
}
