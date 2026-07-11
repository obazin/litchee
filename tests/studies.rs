//! Integration tests for the Studies API.

use futures_util::StreamExt;
use litchee::LichessClient;
use litchee::api::training::studies::StudyChapterMode;
use litchee::model::{LichessColor, PgnExportOptions};
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
async fn export_study_pgn_returns_text() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/study/WTvnkWAL.pgn"))
        .respond_with(ResponseTemplate::new(200).set_body_string("[Event \"Study\"]\n\n1. e4 *"))
        .mount(&server)
        .await;

    let pgn = client(&server)
        .studies()
        .export_study_pgn("WTvnkWAL", &PgnExportOptions::default())
        .await
        .unwrap();

    assert!(pgn.contains("Study"));
}

#[tokio::test]
async fn study_pgn_metadata_reads_last_modified() {
    let server = MockServer::start().await;
    Mock::given(method("HEAD"))
        .and(path("/api/study/WTvnkWAL.pgn"))
        .respond_with(
            ResponseTemplate::new(204)
                .insert_header("Last-Modified", "Tue, 25 Apr 2023 13:23:09 GMT"),
        )
        .mount(&server)
        .await;

    let last_modified = client(&server)
        .studies()
        .study_pgn_metadata("WTvnkWAL")
        .await
        .unwrap();

    assert_eq!(
        last_modified.as_deref(),
        Some("Tue, 25 Apr 2023 13:23:09 GMT")
    );
}

#[tokio::test]
async fn export_chapter_pgn_returns_text() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/study/WTvnkWAL/ch1.pgn"))
        .respond_with(ResponseTemplate::new(200).set_body_string("[Event \"Chapter\"]\n\n1. e4 *"))
        .mount(&server)
        .await;

    let pgn = client(&server)
        .studies()
        .export_chapter_pgn("WTvnkWAL", "ch1", &PgnExportOptions::default())
        .await
        .unwrap();

    assert!(pgn.contains("Chapter"));
}

#[tokio::test]
async fn export_all_pgn_returns_text() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/study/by/bobby/export.pgn"))
        .and(query_param("variations", "false"))
        .and(query_param("orientation", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_string("[Event \"All\"]\n\n1. d4 *"))
        .mount(&server)
        .await;

    let pgn = client(&server)
        .studies()
        .export_all_pgn(
            "bobby",
            &PgnExportOptions::default()
                .variations(false)
                .orientation(true),
        )
        .await
        .unwrap();

    assert!(pgn.contains("All"));
}

#[tokio::test]
async fn list_metadata_streams_studies() {
    let server = MockServer::start().await;
    let body = concat!(
        r#"{"id":"a","name":"One","createdAt":1,"updatedAt":2}"#,
        "\n",
        r#"{"id":"b","name":"Two","createdAt":3,"updatedAt":4}"#,
        "\n",
    );
    Mock::given(method("GET"))
        .and(path("/api/study/by/bobby"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let stream = client(&server)
        .studies()
        .list_metadata("bobby")
        .await
        .unwrap();
    let studies: Vec<_> = stream.collect().await;

    assert_eq!(studies.len(), 2);
    assert_eq!(studies[1].as_ref().unwrap().name, "Two");
}

#[tokio::test]
async fn import_pgn_posts_name_and_pgn() {
    let server = MockServer::start().await;
    let body = r#"{"chapters":[{"id":"ch1","name":"Game 1"}]}"#;
    Mock::given(method("POST"))
        .and(path("/api/study/WTvnkWAL/import-pgn"))
        .and(body_string_contains("name=Game"))
        .and(body_string_contains("pgn="))
        .and(body_string_contains("mode=gamebook"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let result = client(&server)
        .studies()
        .import_pgn("WTvnkWAL", "Game 1", "1. e4 e5 *")
        .orientation(LichessColor::White)
        .mode(StudyChapterMode::Gamebook)
        .send()
        .await
        .unwrap();

    assert_eq!(result.chapters[0].id.as_deref(), Some("ch1"));
}

#[tokio::test]
async fn delete_chapter_sends_delete() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/api/study/WTvnkWAL/ch1"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .mount(&server)
        .await;

    client(&server)
        .studies()
        .delete_chapter("WTvnkWAL", "ch1")
        .await
        .unwrap();
}

#[tokio::test]
async fn create_study_returns_id() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/study"))
        .and(body_string_contains("name=My+Study"))
        .and(body_string_contains("cloneable=nobody"))
        .and(body_string_contains("sticky=false"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"id":"abc12345"}"#))
        .mount(&server)
        .await;
    let id = client(&server)
        .studies()
        .create_study("My Study")
        .visibility("private")
        .cloneable("nobody")
        .sticky(false)
        .send()
        .await
        .unwrap();
    assert_eq!(id, "abc12345");
}

#[tokio::test]
async fn update_chapter_moves_posts_pgn() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/study/s1/c1/moves"))
        .and(body_string_contains("pgn="))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;
    client(&server)
        .studies()
        .update_chapter_moves("s1", "c1", "1. e4 e5 *")
        .await
        .unwrap();
}

#[tokio::test]
async fn update_chapter_tags_posts_pgn() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/study/s1/c1/tags"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;
    client(&server)
        .studies()
        .update_chapter_tags("s1", "c1", "[White \"A\"]")
        .await
        .unwrap();
}
