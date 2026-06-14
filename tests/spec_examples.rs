//! Deserialization tests against Lichess's own example payloads.
//!
//! The JSON fixtures under `tests/fixtures/` are vendored (and converted to
//! strict JSON) from the official `OpenAPI` spec's `examples/` directory. Decoding
//! them through the real client path guards against DTO drift: if our `Lichess*`
//! types stop matching the shapes the API documents, these fail. They use
//! Lichess-authored payloads rather than hand-written minimal mocks.

use litchee::LichessClient;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> LichessClient {
    LichessClient::builder()
        .base_url(&server.uri().parse().expect("mock uri is a valid url"))
        .token("test-token")
        .build()
        .expect("client builds")
}

async fn serve(server: &MockServer, route: &str, body: &str) {
    Mock::given(method("GET"))
        .and(path(route))
        .respond_with(ResponseTemplate::new(200).set_body_string(body.to_owned()))
        .mount(server)
        .await;
}

#[tokio::test]
async fn account_profile_decodes_spec_example() {
    let server = MockServer::start().await;
    serve(
        &server,
        "/api/account",
        include_str!("fixtures/account_profile.json"),
    )
    .await;
    let me = client(&server).account().profile().await.unwrap();
    assert!(!me.user.id.is_empty());
}

#[tokio::test]
async fn user_public_data_decodes_spec_example() {
    let server = MockServer::start().await;
    serve(
        &server,
        "/api/user/anyone",
        include_str!("fixtures/user_public.json"),
    )
    .await;
    let user = client(&server).users().get("anyone").await.unwrap();
    assert!(!user.user.id.is_empty());
}

#[tokio::test]
async fn game_decodes_spec_example() {
    let server = MockServer::start().await;
    serve(
        &server,
        "/api/user/anyone/current-game",
        include_str!("fixtures/game_export.json"),
    )
    .await;
    let game = client(&server)
        .games()
        .current_game("anyone")
        .await
        .unwrap();
    assert!(!game.id.is_empty());
}

#[tokio::test]
async fn team_decodes_spec_example() {
    let server = MockServer::start().await;
    serve(
        &server,
        "/api/team/anyteam",
        include_str!("fixtures/team_single.json"),
    )
    .await;
    let team = client(&server).teams().get("anyteam").await.unwrap();
    assert!(!team.id.is_empty());
}

#[tokio::test]
async fn puzzle_decodes_spec_example() {
    let server = MockServer::start().await;
    serve(
        &server,
        "/api/puzzle/anypuzzle",
        include_str!("fixtures/puzzle_by_id.json"),
    )
    .await;
    // Decoding success is the assertion; a malformed DTO would panic on unwrap.
    let _puzzle = client(&server).puzzles().get("anypuzzle").await.unwrap();
}
