//! Integration tests for the External Engine API.

use futures_util::StreamExt;
use litchee::LichessClient;
use litchee::Secret;
use litchee::api::engine::external_engine::LichessExternalEngineRegistration;
use wiremock::matchers::{body_string_contains, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> LichessClient {
    LichessClient::builder()
        .base_url(&server.uri().parse().expect("mock uri is a valid url"))
        .token("test-token")
        .build()
        .expect("client builds")
}

const ENGINE: &str = r#"{"id":"eng","name":"My Engine","clientSecret":"s","userId":"u",
    "maxThreads":8,"maxHash":256,"variants":["chess"]}"#;

#[tokio::test]
async fn list_returns_engines() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/external-engine"))
        .respond_with(ResponseTemplate::new(200).set_body_string(format!("[{ENGINE}]")))
        .mount(&server)
        .await;

    let engines = client(&server).external_engine().list().await.unwrap();

    assert_eq!(engines.len(), 1);
    assert_eq!(engines[0].max_hash, 256);
}

#[tokio::test]
async fn get_returns_engine() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/external-engine/eng"))
        .respond_with(ResponseTemplate::new(200).set_body_string(ENGINE))
        .mount(&server)
        .await;

    let engine = client(&server).external_engine().get("eng").await.unwrap();

    assert_eq!(engine.id, "eng");
}

#[tokio::test]
async fn update_puts_json_registration() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/api/external-engine/eng"))
        .and(body_string_contains("\"providerSecret\":\"secret\""))
        .respond_with(ResponseTemplate::new(200).set_body_string(ENGINE))
        .mount(&server)
        .await;

    let registration = LichessExternalEngineRegistration {
        name: "My Engine".to_owned(),
        max_threads: 8,
        max_hash: 256,
        provider_secret: Secret::new("secret".to_owned()),
        ..Default::default()
    };
    let engine = client(&server)
        .external_engine()
        .update("eng", &registration)
        .await
        .unwrap();

    assert_eq!(engine.id, "eng");
}

#[tokio::test]
async fn analyse_streams_output() {
    let server = MockServer::start().await;
    let body = concat!(
        r#"{"time":10,"pvs":[]}"#,
        "\n",
        r#"{"time":20,"pvs":[]}"#,
        "\n",
    );
    Mock::given(method("POST"))
        .and(path("/api/external-engine/eng/analyse"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;
    let client = LichessClient::builder()
        .engine_url(&server.uri().parse().unwrap())
        .token("t")
        .build()
        .unwrap();
    let work = serde_json::json!({"sessionId":"s","threads":1});
    let stream = client
        .external_engine()
        .analyse("eng", "client-secret", &work)
        .await
        .unwrap();
    let items: Vec<_> = stream.collect().await;
    assert_eq!(items.len(), 2);
}

#[tokio::test]
async fn submit_work_posts_output() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/external-engine/work/w1"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;
    let client = LichessClient::builder()
        .engine_url(&server.uri().parse().unwrap())
        .token("t")
        .build()
        .unwrap();
    client
        .external_engine()
        .submit_work("w1", "info depth 1\nbestmove e2e4\n")
        .await
        .unwrap();
}

#[tokio::test]
async fn create_posts_json_registration() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/external-engine"))
        .and(body_string_contains("\"providerSecret\":\"secret\""))
        .respond_with(ResponseTemplate::new(200).set_body_string(ENGINE))
        .mount(&server)
        .await;

    let registration = LichessExternalEngineRegistration {
        name: "My Engine".to_owned(),
        max_threads: 8,
        max_hash: 256,
        provider_secret: Secret::new("secret".to_owned()),
        ..Default::default()
    };
    let engine = client(&server)
        .external_engine()
        .create(&registration)
        .await
        .unwrap();

    assert_eq!(engine.id, "eng");
}

#[tokio::test]
async fn delete_removes_an_engine() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/api/external-engine/eng"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;

    client(&server)
        .external_engine()
        .delete("eng")
        .await
        .unwrap();
}

#[tokio::test]
async fn acquire_work_returns_none_on_204() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/external-engine/work"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;
    // point the engine host at the mock server
    let client = LichessClient::builder()
        .engine_url(&server.uri().parse().unwrap())
        .token("t")
        .build()
        .unwrap();
    let work = client
        .external_engine()
        .acquire_work("secret")
        .await
        .unwrap();
    assert!(work.is_none());
}

#[tokio::test]
async fn acquire_work_returns_work_on_200() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/external-engine/work"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"id":"w1","sessionId":"s"}"#))
        .mount(&server)
        .await;
    let client = LichessClient::builder()
        .engine_url(&server.uri().parse().unwrap())
        .token("t")
        .build()
        .unwrap();
    let work = client
        .external_engine()
        .acquire_work("secret")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(work.id.as_deref(), Some("w1"));
}
