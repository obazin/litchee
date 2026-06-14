//! Opt-in retry of rate-limited (`429`) requests.

use std::time::Duration;

use litchee::error::{ApiErrorKind, LichessError};
use litchee::{LichessClient, RetryPolicy};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const PROFILE: &str = r#"{"id":"me","username":"Me","url":"https://lichess.org/@/Me"}"#;

fn client(server: &MockServer, policy: Option<RetryPolicy>) -> LichessClient {
    let mut builder = LichessClient::builder()
        .base_url(&server.uri().parse().expect("mock uri is a valid url"))
        .token("test-token");
    if let Some(policy) = policy {
        builder = builder.retry_policy(policy);
    }
    builder.build().expect("client builds")
}

#[tokio::test]
async fn retries_a_rate_limited_request_then_succeeds() {
    let server = MockServer::start().await;
    // First call: 429 with Retry-After: 0 (so the retry is immediate). Higher
    // priority + up_to_n_times(1) so it wins exactly once.
    Mock::given(method("GET"))
        .and(path("/api/account"))
        .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "0"))
        .up_to_n_times(1)
        .with_priority(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/account"))
        .respond_with(ResponseTemplate::new(200).set_body_string(PROFILE))
        .with_priority(5)
        .mount(&server)
        .await;

    let policy = RetryPolicy::new(2).with_base_delay(Duration::from_millis(1));
    let me = client(&server, Some(policy))
        .account()
        .profile()
        .await
        .expect("retry should recover from the 429");

    assert_eq!(me.user.id, "me");
}

#[tokio::test]
async fn without_a_policy_a_429_is_surfaced() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/account"))
        .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "1"))
        .mount(&server)
        .await;

    let err = client(&server, None).account().profile().await.unwrap_err();

    assert!(
        matches!(
            &err,
            LichessError::Api(api)
                if matches!(api.kind, ApiErrorKind::RateLimited { retry_after_secs: Some(1) })
        ),
        "expected a RateLimited error carrying Retry-After, got {err:?}"
    );
}

#[tokio::test]
async fn gives_up_after_exhausting_retries() {
    let server = MockServer::start().await;
    // Always 429: the policy retries, then the final 429 is surfaced.
    Mock::given(method("GET"))
        .and(path("/api/account"))
        .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "0"))
        .mount(&server)
        .await;

    let policy = RetryPolicy::new(2).with_base_delay(Duration::from_millis(1));
    let err = client(&server, Some(policy))
        .account()
        .profile()
        .await
        .unwrap_err();

    assert!(
        matches!(&err, LichessError::Api(api) if matches!(api.kind, ApiErrorKind::RateLimited { .. })),
        "expected the final 429 to surface, got {err:?}"
    );
}
