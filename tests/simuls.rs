//! Integration tests for the Simuls API.

use litchee::LichessClient;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> LichessClient {
    LichessClient::builder()
        .base_url(&server.uri().parse().expect("mock uri is a valid url"))
        .build()
        .expect("client builds")
}

#[tokio::test]
async fn current_returns_grouped_simuls() {
    let server = MockServer::start().await;
    let body = r#"{
        "started":[{
            "id":"abc","name":"Sunday","fullName":"Sunday Simul",
            "host":{"id":"bobby","name":"Bobby","rating":2400},
            "variants":[{"key":"standard","name":"Standard"}],
            "isCreated":false,"isFinished":false,"isRunning":true,
            "nbApplicants":3,"nbPairings":5
        }],
        "created":[],"finished":[]
    }"#;
    Mock::given(method("GET"))
        .and(path("/api/simul"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let simuls = client(&server).simuls().current().await.unwrap();

    assert_eq!(simuls.started.len(), 1);
    assert_eq!(simuls.started[0].host.user.name, "Bobby");
    assert!(simuls.started[0].is_running);
    assert!(simuls.pending.is_empty());
}
