//! Integration tests for the `OAuth2` PKCE flow.

use std::collections::HashMap;

use litchee::LichessClient;
use litchee::api::auth::oauth::{AuthorizationRequest, CodeExchange, PkceVerifier, Scope};
use litchee::error::{LichessError, OAuthErrorCode};
use wiremock::matchers::{body_string_contains, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> LichessClient {
    LichessClient::builder()
        .base_url(&server.uri().parse().expect("mock uri is a valid url"))
        .token("test-token")
        .build()
        .expect("client builds")
}

#[test]
fn authorization_url_contains_all_pkce_parameters() {
    let client = LichessClient::builder()
        .base_url(&"https://lichess.org".parse().unwrap())
        .build()
        .unwrap();
    let scopes = [Scope::BoardPlay, Scope::ChallengeWrite];
    let request = AuthorizationRequest {
        client_id: "my-app",
        redirect_uri: "https://my.app/callback",
        scopes: &scopes,
        username_hint: Some("bobby"),
    };

    let auth = client.oauth().authorization_url(&request).unwrap();

    assert_eq!(auth.url.scheme(), "https");
    assert_eq!(auth.url.host_str(), Some("lichess.org"));
    assert_eq!(auth.url.path(), "/oauth");
    let params: HashMap<_, _> = auth.url.query_pairs().into_owned().collect();
    assert_eq!(params["response_type"], "code");
    assert_eq!(params["client_id"], "my-app");
    assert_eq!(params["redirect_uri"], "https://my.app/callback");
    assert_eq!(params["code_challenge_method"], "S256");
    assert_eq!(params["scope"], "board:play challenge:write");
    assert_eq!(params["state"], auth.state);
    assert_eq!(params["username"], "bobby");
    assert!(!params["code_challenge"].is_empty());
}

#[tokio::test]
async fn exchange_code_returns_a_token() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/token"))
        .and(body_string_contains("grant_type=authorization_code"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"{"token_type":"Bearer","access_token":"lio_abc","expires_in":31536000}"#,
        ))
        .mount(&server)
        .await;
    let verifier = PkceVerifier::generate();
    let exchange = CodeExchange {
        code: "auth-code",
        code_verifier: &verifier,
        redirect_uri: "https://my.app/callback",
        client_id: "my-app",
    };

    let token = client(&server)
        .oauth()
        .exchange_code(&exchange)
        .await
        .unwrap();

    assert_eq!(token.access_token.expose(), "lio_abc");
    assert_eq!(token.token_type, "Bearer");
}

#[tokio::test]
async fn failed_exchange_maps_to_typed_oauth_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/token"))
        .respond_with(ResponseTemplate::new(400).set_body_string(
            r#"{"error":"invalid_grant","error_description":"hash does not match"}"#,
        ))
        .mount(&server)
        .await;
    let verifier = PkceVerifier::generate();
    let exchange = CodeExchange {
        code: "bad-code",
        code_verifier: &verifier,
        redirect_uri: "https://my.app/callback",
        client_id: "my-app",
    };

    let error = client(&server)
        .oauth()
        .exchange_code(&exchange)
        .await
        .unwrap_err();

    match error {
        LichessError::OAuth(oauth) => {
            assert_eq!(oauth.code, OAuthErrorCode::InvalidGrant);
            assert_eq!(oauth.description.as_deref(), Some("hash does not match"));
        }
        other => panic!("expected an OAuth error, got {other:?}"),
    }
}

#[tokio::test]
async fn revoke_token_succeeds_on_no_content() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/api/token"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    client(&server).oauth().revoke_token().await.unwrap();
}

#[tokio::test]
async fn test_tokens_returns_map() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/token/test"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"{"tok1":{"userId":"bob","scopes":"board:play"},"tok2":null}"#),
        )
        .mount(&server)
        .await;
    let map = client(&server)
        .oauth()
        .test_tokens(&["tok1", "tok2"])
        .await
        .unwrap();
    assert!(map["tok1"].is_object());
    assert!(map["tok2"].is_null());
}
