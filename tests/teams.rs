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

#[tokio::test]
async fn of_user_returns_teams() {
    let server = MockServer::start().await;
    let body = r#"[{"id":"coders","name":"Coders"},{"id":"chess","name":"Chess"}]"#;
    Mock::given(method("GET"))
        .and(path("/api/team/of/mary"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let teams = client(&server).teams().of_user("mary").await.unwrap();

    assert_eq!(teams.len(), 2);
    assert_eq!(teams[0].id, "coders");
}

#[tokio::test]
async fn search_paginates() {
    let server = MockServer::start().await;
    let body = r#"{"currentPage":1,"maxPerPage":15,"currentPageResults":[],
        "previousPage":null,"nextPage":2,"nbResults":30,"nbPages":2}"#;
    Mock::given(method("GET"))
        .and(path("/api/team/search"))
        .and(query_param("text", "chess"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let page = client(&server).teams().search("chess", 1).await.unwrap();

    assert_eq!(page.next_page, Some(2));
}

#[tokio::test]
async fn quit_posts_to_the_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/team/coders/quit"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;

    client(&server).teams().quit("coders").await.unwrap();
}

#[tokio::test]
async fn kick_posts_to_the_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/team/coders/kick/mary"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;

    client(&server)
        .teams()
        .kick("coders", "mary")
        .await
        .unwrap();
}

#[tokio::test]
async fn message_all_posts_the_message() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/team/coders/pm-all"))
        .and(body_string_contains("message=hello"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;

    client(&server)
        .teams()
        .message_all("coders", "hello")
        .await
        .unwrap();
}

#[tokio::test]
async fn join_requests_returns_requests() {
    let server = MockServer::start().await;
    let body = r#"[{"request":{"teamId":"coders","userId":"mary","date":1,"message":"hi"},
        "user":{"id":"mary","username":"Mary"}}]"#;
    Mock::given(method("GET"))
        .and(path("/api/team/coders/requests"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let requests = client(&server)
        .teams()
        .join_requests("coders")
        .await
        .unwrap();

    assert_eq!(requests[0].request.user_id, "mary");
}

#[tokio::test]
async fn decline_request_posts_to_the_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/team/coders/request/mary/decline"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;

    client(&server)
        .teams()
        .decline_request("coders", "mary")
        .await
        .unwrap();
}

#[tokio::test]
async fn swiss_tournaments_streams() {
    let server = MockServer::start().await;
    let body = "{\"id\":\"a\"}\n{\"id\":\"b\"}\n";
    Mock::given(method("GET"))
        .and(path("/api/team/coders/swiss"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let stream = client(&server)
        .teams()
        .swiss_tournaments("coders")
        .await
        .unwrap();
    let swisses: Vec<_> = stream.collect().await;

    assert_eq!(swisses.len(), 2);
}
