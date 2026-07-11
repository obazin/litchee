//! Smoke tests against the **real** Lichess API.
//!
//! These are `#[ignore]`d so the normal (offline, deterministic) test run is
//! unaffected. They hit the public internet to catch drift that mocked tests
//! cannot — a DTO that no longer matches a live response. Run them explicitly:
//!
//! ```text
//! cargo test --test live -- --ignored --nocapture
//! ```
//!
//! Most cover stable, unauthenticated public endpoints and need no credentials.
//! The authenticated check is skipped unless `LICHESS_TOKEN` is set in the env.
//!
//! The opening-explorer host (`explorer.lichess.org`) is intentionally not
//! covered here: it returns an nginx `401` to non-browser/datacenter clients
//! (verified directly with `curl`), so a live check would fail in the very
//! environments these run in. Its DTOs are exercised by the mocked tests.

use litchee::LichessClient;
use litchee::api::users::players::UserQuery;

/// The Lichess founder's account — a permanent, stable public profile.
const STABLE_USER: &str = "thibault";
/// Magnus Carlsen's FIDE id — a stable record in the FIDE database.
const CARLSEN_FIDE_ID: u32 = 1_503_014;
/// A King+Queen vs King position (white to move) — a tablebase win.
const KQK_FEN: &str = "4k3/8/8/8/8/8/8/4KQ2 w - - 0 1";

#[tokio::test]
#[ignore = "hits the live Lichess API; run with --ignored"]
async fn user_public_profile() {
    let user = LichessClient::new()
        .users()
        .get(STABLE_USER, &UserQuery::default())
        .await
        .expect("fetch public profile");
    assert_eq!(user.user.id, STABLE_USER);
    assert!(user.user.perfs.is_some(), "real profile should carry perfs");
}

#[tokio::test]
#[ignore = "hits the live Lichess API; run with --ignored"]
async fn users_status() {
    let statuses = LichessClient::new()
        .users()
        .statuses(&[STABLE_USER], None, None, None)
        .await
        .expect("fetch user status");
    assert_eq!(statuses.first().expect("one status").user.id, STABLE_USER);
}

#[tokio::test]
#[ignore = "hits the live Lichess API; run with --ignored"]
async fn daily_puzzle() {
    let daily = LichessClient::new()
        .puzzles()
        .daily()
        .await
        .expect("fetch daily puzzle");
    assert!(!daily.puzzle.id.is_empty());
    assert!(!daily.game.id.is_empty());
}

#[tokio::test]
#[ignore = "hits the live Lichess API; run with --ignored"]
async fn fide_player() {
    let player = LichessClient::new()
        .fide()
        .get(CARLSEN_FIDE_ID)
        .await
        .expect("fetch FIDE player");
    assert_eq!(player.id, CARLSEN_FIDE_ID);
    assert!(
        player.name.to_lowercase().contains("carlsen"),
        "unexpected name: {}",
        player.name
    );
}

#[tokio::test]
#[ignore = "hits the live Lichess API; run with --ignored"]
async fn tablebase_standard() {
    let position = LichessClient::new()
        .tablebase()
        .standard(KQK_FEN, None)
        .await
        .expect("query tablebase");
    assert!(!position.moves.is_empty(), "KQ vs K has legal moves");
}

#[tokio::test]
#[ignore = "hits the live Lichess API; needs LICHESS_TOKEN; run with --ignored"]
async fn authenticated_account_profile() {
    // Treat an empty value as unset: in CI an undefined secret still expands to
    // an empty string rather than being absent.
    let Some(token) = std::env::var("LICHESS_TOKEN")
        .ok()
        .filter(|t| !t.is_empty())
    else {
        eprintln!("skipping: LICHESS_TOKEN not set");
        return;
    };
    let me = LichessClient::builder()
        .token(token)
        .build()
        .expect("client builds")
        .account()
        .profile()
        .await
        .expect("fetch own profile");
    assert!(!me.user.id.is_empty());
}
