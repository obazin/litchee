//! Integration tests for the Analysis API.

use litchee::LichessClient;
use litchee::model::LichessVariantKey;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> LichessClient {
    LichessClient::builder()
        .base_url(&server.uri().parse().expect("mock uri is a valid url"))
        .build()
        .expect("client builds")
}

#[tokio::test]
async fn cloud_eval_sends_parameters_and_parses_pvs() {
    let server = MockServer::start().await;
    let body = r#"{"fen":"startpos","knodes":100,"depth":20,
                   "pvs":[{"moves":"e2e4 e7e5","cp":20}]}"#;
    Mock::given(method("GET"))
        .and(path("/api/cloud-eval"))
        .and(query_param("fen", "the-fen"))
        .and(query_param("multiPv", "3"))
        .and(query_param("variant", "standard"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let eval = client(&server)
        .analysis()
        .cloud_eval("the-fen")
        .multi_pv(3)
        .variant(LichessVariantKey::Standard)
        .send()
        .await
        .unwrap();

    assert_eq!(eval.depth, 20);
    assert_eq!(eval.pvs[0].cp, Some(20));
}

#[tokio::test]
async fn cloud_eval_missing_position_is_not_found() {
    use litchee::error::{ApiErrorKind, LichessError};

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/cloud-eval"))
        .respond_with(ResponseTemplate::new(404).set_body_string(r#"{"error":"No eval"}"#))
        .mount(&server)
        .await;

    let error = client(&server)
        .analysis()
        .cloud_eval("missing")
        .send()
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        LichessError::Api(api) if api.kind == ApiErrorKind::NotFound
    ));
}
