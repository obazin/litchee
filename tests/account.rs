//! Integration tests for the Account API, served by a mock HTTP server.

use litchee::LichessClient;
use litchee::error::{ApiErrorKind, LichessError};
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Builds a client pointed at the mock server with a bearer token set.
fn client(server: &MockServer) -> LichessClient {
    LichessClient::builder()
        .base_url(&server.uri().parse().expect("mock uri is a valid url"))
        .token("test-token")
        .build()
        .expect("client builds")
}

/// A trimmed but valid `UserExtended` body, derived from the spec example.
const PROFILE_BODY: &str = r#"{
    "id": "bobby",
    "username": "Bobby",
    "perfs": { "blitz": { "games": 109, "rating": 1814, "rd": 55, "prog": -19 } },
    "createdAt": 1775598473954,
    "playTime": { "total": 19991, "tv": 0 },
    "url": "https://lichess.org/@/Bobby",
    "count": { "all": 2054, "rated": 1643, "draw": 231, "loss": 921, "win": 902,
               "bookmark": 0, "playing": 0, "import": 0, "me": 0 },
    "followable": true,
    "following": false,
    "blocking": false
}"#;

#[tokio::test]
async fn profile_returns_extended_user_and_sends_bearer_token() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/account"))
        .and(header("authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_string(PROFILE_BODY))
        .mount(&server)
        .await;

    let user = client(&server).account().profile().await.unwrap();

    assert_eq!(user.user.id, "bobby");
    assert_eq!(user.url, "https://lichess.org/@/Bobby");
    assert_eq!(user.count.unwrap().all, 2054);
    assert_eq!(user.following, Some(false));
}

#[tokio::test]
async fn email_extracts_the_address() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/account/email"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"email":"abc@example.com"}"#))
        .mount(&server)
        .await;

    let email = client(&server).account().email().await.unwrap();

    assert_eq!(email, "abc@example.com");
}

#[tokio::test]
async fn kid_mode_reads_the_flag() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/account/kid"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"kid":true}"#))
        .mount(&server)
        .await;

    assert!(client(&server).account().kid_mode().await.unwrap());
}

#[tokio::test]
async fn set_kid_mode_sends_the_value_and_succeeds() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/account/kid"))
        .and(query_param("v", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;

    client(&server).account().set_kid_mode(true).await.unwrap();
}

#[tokio::test]
async fn unauthorized_profile_maps_to_typed_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/account"))
        .respond_with(ResponseTemplate::new(401).set_body_string(r#"{"error":"No such token"}"#))
        .mount(&server)
        .await;

    let error = client(&server).account().profile().await.unwrap_err();

    match error {
        LichessError::Api(api) => {
            assert_eq!(api.kind, ApiErrorKind::Unauthorized);
            assert_eq!(api.message.as_deref(), Some("No such token"));
        }
        other => panic!("expected an API error, got {other:?}"),
    }
}

#[tokio::test]
async fn preferences_returns_typed_and_extra_fields() {
    let server = MockServer::start().await;
    let body = r#"{"prefs":{"theme":"blue","zen":1},"language":"en-GB"}"#;
    Mock::given(method("GET"))
        .and(path("/api/account/preferences"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;
    let prefs = client(&server).account().preferences().await.unwrap();
    assert_eq!(prefs.language.as_deref(), Some("en-GB"));
    assert_eq!(prefs.prefs.theme.as_deref(), Some("blue"));
}

#[tokio::test]
async fn timeline_returns_entries() {
    let server = MockServer::start().await;
    let body = r#"{"entries":[{"type":"follow","date":1}],"users":{}}"#;
    Mock::given(method("GET"))
        .and(path("/api/timeline"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;
    let timeline = client(&server).account().timeline().await.unwrap();
    assert_eq!(timeline.entries[0].entry_type, "follow");
}
