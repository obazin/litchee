//! Integration tests proving caller-supplied path segments are percent-encoded
//! so a value containing `/`, `?`, or `#` cannot reshape the request path.

use litchee::LichessClient;
use litchee::api::users::players::UserQuery;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> LichessClient {
    LichessClient::builder()
        .base_url(&server.uri().parse().expect("mock uri is a valid url"))
        .build()
        .expect("client builds")
}

#[tokio::test]
async fn slash_in_username_is_encoded_not_treated_as_separator() {
    let server = MockServer::start().await;
    let body = r#"{"id":"x","username":"x","url":"https://lichess.org/@/x"}"#;
    // The matcher only fires on the single encoded segment. If the slash leaked
    // through raw, the request path would be `/api/user/a/b` and never match.
    Mock::given(method("GET"))
        .and(path("/api/user/a%2Fb"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let user = client(&server)
        .users()
        .get("a/b", &UserQuery::default())
        .await;

    assert!(user.is_ok(), "encoded path should match the mounted route");
}

#[tokio::test]
async fn query_and_fragment_characters_in_segment_are_encoded() {
    let server = MockServer::start().await;
    let body = r#"{"id":"x","username":"x","url":"https://lichess.org/@/x"}"#;
    Mock::given(method("GET"))
        .and(path("/api/user/a%3Fb%23c"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let user = client(&server)
        .users()
        .get("a?b#c", &UserQuery::default())
        .await;

    assert!(user.is_ok(), "`?`/`#` must not split off a query/fragment");
}
