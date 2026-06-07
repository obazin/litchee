//! Integration tests for the Teams API.

use futures_util::StreamExt;
use litchee::LichessClient;
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
async fn get_returns_a_team() {
    let server = MockServer::start().await;
    let body = r#"{"id":"coders","name":"Coders","nbMembers":42}"#;
    Mock::given(method("GET"))
        .and(path("/api/team/coders"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let team = client(&server).teams().get("coders").await.unwrap();

    assert_eq!(team.nb_members, Some(42));
}

#[tokio::test]
async fn all_paginates() {
    let server = MockServer::start().await;
    let body = r#"{"currentPage":1,"maxPerPage":15,"currentPageResults":[],
        "previousPage":null,"nextPage":2,"nbResults":30,"nbPages":2}"#;
    Mock::given(method("GET"))
        .and(path("/api/team/all"))
        .and(query_param("page", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let page = client(&server).teams().all(1).await.unwrap();

    assert_eq!(page.nb_pages, 2);
}

#[tokio::test]
async fn members_streams_users() {
    let server = MockServer::start().await;
    let body = "{\"id\":\"a\",\"username\":\"A\"}\n{\"id\":\"b\",\"username\":\"B\"}\n";
    Mock::given(method("GET"))
        .and(path("/api/team/coders/users"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let stream = client(&server).teams().members("coders").await.unwrap();
    let members: Vec<_> = stream.collect().await;

    assert_eq!(members.len(), 2);
}

#[tokio::test]
async fn join_posts_message_and_password() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/team/coders/join"))
        .and(body_string_contains("message=hi"))
        .and(body_string_contains("password=secret"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;

    client(&server)
        .teams()
        .join("coders", Some("hi"), Some("secret"))
        .await
        .unwrap();
}

#[tokio::test]
async fn accept_request_posts_to_the_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/team/coders/request/mary/accept"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;

    client(&server)
        .teams()
        .accept_request("coders", "mary")
        .await
        .unwrap();
}

#[tokio::test]
async fn arena_tournaments_streams() {
    let server = MockServer::start().await;
    let body = "{\"id\":\"a\"}\n{\"id\":\"b\"}\n";
    Mock::given(method("GET"))
        .and(path("/api/team/coders/arena"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;
    let stream = client(&server)
        .teams()
        .arena_tournaments("coders")
        .await
        .unwrap();
    let arenas: Vec<_> = stream.collect().await;
    assert_eq!(arenas.len(), 2);
}
