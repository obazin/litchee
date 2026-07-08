# litchee

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust 2024](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org)
[![Lichess API](https://img.shields.io/badge/Lichess%20API-184%2F184%20operations-brightgreen.svg)](https://lichess.org/api)

**`litchee`** is an asynchronous, builder-pattern Rust client for the
[Lichess API], covering **every documented operation** (184/184 of the official
OpenAPI spec) — from looking up players to playing games, running tournaments,
streaming live broadcasts, and "Log in with Lichess" via OAuth2 + PKCE.

It is open source (MIT) and aims at **feature parity** with the official API,
with an ergonomic, strongly-typed surface: every DTO is prefixed `Lichess*`,
every failure maps to a specific [`LichessError`] variant, and every endpoint is
reached through a small accessor on the client (`client.account()`,
`client.broadcasts()`, …).

```rust,no_run
use futures_util::StreamExt;
use litchee::LichessClient;

#[tokio::main]
async fn main() -> litchee::Result<()> {
    let client = LichessClient::builder().token("lip_your_token").build()?;

    // A simple JSON request.
    let me = client.account().profile().await?;
    println!("Logged in as {}", me.user.username);

    // A streaming (NDJSON) request.
    let mut games = client.games().export_user("bobby").max(5).stream().await?;
    while let Some(game) = games.next().await {
        println!("game {}", game?.id);
    }
    Ok(())
}
```

---

## Why litchee?

The Lichess API is large (24 tags, ~184 operations, four hosts, JSON + NDJSON +
PGN). `litchee` wraps all of it behind one cohesive, async-first client so Rust
applications — bots, analysis tools, "Log in with Lichess" web apps, dataset
exporters — can talk to Lichess without hand-rolling HTTP, streaming, and OAuth.

- **Complete.** All 184 documented operations are implemented and covered by
  tests.
- **Async & streaming-native.** Built on `tokio` + `reqwest`; NDJSON endpoints
  (event streams, board/bot game state, game exports) return a `Stream` you can
  consume directly.
- **Typed end to end.** `Lichess*` DTOs and an exhaustive, matchable error type.
- **"Log in with Lichess".** First-class OAuth2 Authorization Code flow with
  PKCE, plus plain personal access tokens.
- **Organized by business concern.** The module tree mirrors the API's own
  structure, grouped into categories under `litchee::api`.

## Installation

```toml
[dependencies]
litchee = "0.1"
tokio = { version = "1", features = ["full"] }
futures-util = "0.3" # to consume streams with `.next()`
```

The minimum supported Rust version is **1.95** (edition 2024).

## Examples

### 1. Log in with Lichess (OAuth2 + PKCE)

> New to OAuth or PKCE? The [**PKCE flow guide**](PKCE_GUIDE.md) walks through the
> whole "Log in with Lichess" flow step by step, for beginners — with a glossary
> of OAuth terms at the end.

```rust,no_run
use litchee::LichessClient;
use litchee::api::auth::oauth::{AuthorizationRequest, CodeExchange, Scope};

# async fn run() -> litchee::Result<()> {
let client = LichessClient::new();

// 1. Build the authorization URL; persist `state` and `verifier` in the session.
let auth = client.oauth().authorization_url(&AuthorizationRequest {
    client_id: "your.app",
    redirect_uri: "https://your.app/callback",
    scopes: &[Scope::PreferenceRead, Scope::PuzzleRead, Scope::StudyRead],
    username_hint: None,
})?;
println!("Send the user to: {}", auth.url);

// 2. After the redirect, check `state`, then exchange the returned `code`.
let token = client.oauth().exchange_code(&CodeExchange {
    code: "code_from_redirect",
    code_verifier: &auth.verifier,
    redirect_uri: "https://your.app/callback",
    client_id: "your.app",
}).await?;

// 3. Build an authenticated client with the new token.
let user = LichessClient::builder().token(token.access_token.into_inner()).build()?;
println!("Hello, {}", user.account().profile().await?.user.username);
# Ok(())
# }
```

### 2. Export an authenticated user's games (NDJSON stream)

```rust,no_run
use futures_util::StreamExt;
use litchee::LichessClient;

# async fn run() -> litchee::Result<()> {
let client = LichessClient::builder().token("lip_your_token").build()?;
let me = client.account().profile().await?;

// Stream this user's last 20 rated blitz games as decoded JSON.
let mut games = client
    .games()
    .export_user(&me.user.username)
    .max(20)
    .rated(true)
    .perf_type("blitz")
    .stream()
    .await?;

while let Some(game) = games.next().await {
    let game = game?;
    println!("{} — winner: {:?}", game.id, game.winner);
}
# Ok(())
# }
```

### 3. A user's played puzzles

```rust,no_run
use futures_util::StreamExt;
use litchee::LichessClient;

# async fn run() -> litchee::Result<()> {
let client = LichessClient::builder().token("lip_your_token").build()?;

// Stream the authenticated user's puzzle history (needs the `puzzle:read` scope).
let mut activity = client.puzzles().activity(Some(50), None, None).await?;
while let Some(round) = activity.next().await {
    let round = round?;
    let outcome = if round.win { "solved" } else { "failed" };
    println!("puzzle {} — {outcome}", round.puzzle.id);
}
# Ok(())
# }
```

### 4. Studies (list + export PGN)

```rust,no_run
use futures_util::StreamExt;
use litchee::LichessClient;

# async fn run() -> litchee::Result<()> {
let client = LichessClient::builder().token("lip_your_token").build()?;

// List a user's studies, then export the first one as PGN.
let mut studies = client.studies().list_metadata("bobby").await?;
if let Some(study) = studies.next().await {
    let study = study?;
    let pgn = client
        .studies()
        .export_study_pgn(&study.id, &Default::default())
        .await?;
    println!("{} — {} bytes of PGN", study.name, pgn.len());
}
# Ok(())
# }
```

### 5. Broadcasting (browse + round PGN)

```rust,no_run
use futures_util::StreamExt;
use litchee::LichessClient;

# async fn run() -> litchee::Result<()> {
let client = LichessClient::new();

// Browse official broadcasts, then export a round's games as PGN.
let mut official = client.broadcasts().official().await?;
if let Some(broadcast) = official.next().await {
    let broadcast = broadcast?;
    println!("Broadcast: {}", broadcast.tour.name);
    if let Some(round) = broadcast.rounds.first() {
        let pgn = client
            .broadcasts()
            .round_pgn(&round.id, &Default::default())
            .await?;
        println!("Round '{}' — {} bytes of PGN", round.name, pgn.len());
    }
}
# Ok(())
# }
```

The [`examples/`](examples/) directory contains runnable programs:

- [`oauth_flow`](examples/oauth_flow.rs) — the **full "Log in with Lichess"
  flow** end to end: PKCE authorization, opening the browser, catching the
  redirect on a tiny local listener, exchanging the code, then listing the
  signed-in user's recent games, puzzle attempts, and studies
  (`cargo run --example oauth_flow`).
- [`profile`](examples/profile.rs) — print the authenticated user's profile
  (`LICHESS_TOKEN=lip_xxx cargo run --example profile`).
- [`tv_feed`](examples/tv_feed.rs) — stream the Lichess TV feed
  (`cargo run --example tv_feed`).

## API coverage

Every documented Lichess operation is implemented. Endpoints are reached through
an accessor method on `LichessClient` and live in a module under `litchee::api`,
grouped by category. The tables below map each concern's endpoints to its module.

<details>
<summary><b>Auth</b></summary>

**`client.oauth()`** — `litchee::api::auth::oauth`  (4 endpoints)

`GET /oauth`, `POST /api/token`, `DELETE /api/token`, `POST /api/token/test`

</details>

<details>
<summary><b>Users</b></summary>

**`client.account()`** — `litchee::api::users::account`  (6 endpoints)

`GET /api/account`, `GET /api/account/email`, `GET /api/account/preferences`, `GET /api/account/kid`, `POST /api/account/kid`, `GET /api/timeline`

**`client.users()`** — `litchee::api::users::players`  (13 endpoints)

`GET /api/user/{username}`, `POST /api/users`, `GET /api/users/status`, `GET /api/crosstable/{u1}/{u2}`, `GET /api/player/autocomplete`, `GET /api/user/{username}/rating-history`, `GET /api/user/{username}/perf/{perf}`, `GET /api/user/{username}/activity`, `GET /api/player`, `GET /api/player/top/{nb}/{perfType}`, `GET /api/streamer/live`, `GET /api/user/{username}/note`, `POST /api/user/{username}/note`

**`client.fide()`** — `litchee::api::users::fide`  (3 endpoints)

`GET /api/fide/player/{playerId}`, `GET /api/fide/player/{playerId}/ratings`, `GET /api/fide/player`

</details>

<details>
<summary><b>Social</b></summary>

**`client.relations()`** — `litchee::api::social::relations`  (5 endpoints)

`GET /api/rel/following`, `POST /api/rel/follow/{username}`, `POST /api/rel/unfollow/{username}`, `POST /api/rel/block/{username}`, `POST /api/rel/unblock/{username}`

**`client.messaging()`** — `litchee::api::social::messaging`  (1 endpoint)

`POST /inbox/{username}`

**`client.teams()`** — `litchee::api::social::teams`  (14 endpoints)

`GET /api/team/{teamId}`, `GET /api/team/all`, `GET /api/team/of/{username}`, `GET /api/team/search`, `GET /api/team/{teamId}/users`, `GET /api/team/{teamId}/requests`, `POST /api/team/{teamId}/request/{userId}/accept`, `POST /api/team/{teamId}/request/{userId}/decline`, `POST /api/team/{teamId}/kick/{userId}`, `POST /team/{teamId}/join`, `POST /team/{teamId}/quit`, `POST /team/{teamId}/pm-all`, `GET /api/team/{teamId}/arena`, `GET /api/team/{teamId}/swiss`

</details>

<details>
<summary><b>Tournaments</b></summary>

**`client.arena()`** — `litchee::api::tournaments::arena`  (13 endpoints)

`GET /api/tournament`, `GET /api/tournament/{id}`, `POST /api/tournament`, `POST /api/tournament/{id}`, `POST /api/tournament/team-battle/{id}`, `GET /api/tournament/{id}/teams`, `POST /api/tournament/{id}/join`, `POST /api/tournament/{id}/withdraw`, `POST /api/tournament/{id}/terminate`, `GET /api/tournament/{id}/results`, `GET /api/tournament/{id}/games`, `GET /api/user/{username}/tournament/created`, `GET /api/user/{username}/tournament/played`

**`client.swiss()`** — `litchee::api::tournaments::swiss`  (10 endpoints)

`GET /api/swiss/{id}`, `POST /api/swiss/new/{teamId}`, `POST /api/swiss/{id}/edit`, `POST /api/swiss/{id}/join`, `POST /api/swiss/{id}/withdraw`, `POST /api/swiss/{id}/terminate`, `POST /api/swiss/{id}/schedule-next-round`, `GET /swiss/{id}.trf`, `GET /api/swiss/{id}/results`, `GET /api/swiss/{id}/games`

**`client.simuls()`** — `litchee::api::tournaments::simuls`  (1 endpoint)

`GET /api/simul`

</details>

<details>
<summary><b>Training</b></summary>

**`client.puzzles()`** — `litchee::api::training::puzzles`  (11 endpoints)

`GET /api/puzzle/daily`, `GET /api/puzzle/{id}`, `GET /api/puzzle/next`, `GET /api/puzzle/activity`, `GET /api/puzzle/batch/{angle}`, `POST /api/puzzle/batch/{angle}`, `GET /api/puzzle/dashboard/{days}`, `GET /api/puzzle/replay/{days}/{theme}`, `GET /api/storm/dashboard/{username}`, `GET /api/racer/{id}`, `POST /api/racer`

**`client.studies()`** — `litchee::api::training::studies`  (9 endpoints)

`GET /api/study/{studyId}/{chapterId}.pgn`, `GET /api/study/{studyId}.pgn`, `GET /api/study/by/{username}/export.pgn`, `GET /api/study/by/{username}`, `POST /api/study`, `POST /api/study/{studyId}/import-pgn`, `POST /api/study/{studyId}/{chapterId}/moves`, `POST /api/study/{studyId}/{chapterId}/tags`, `DELETE /api/study/{studyId}/{chapterId}`

</details>

<details>
<summary><b>Broadcasting</b></summary>

**`client.broadcasts()`** — `litchee::api::broadcasting::broadcasts`  (19 endpoints)

`GET /api/broadcast`, `GET /api/broadcast/top`, `GET /api/broadcast/search`, `GET /api/broadcast/by/{username}`, `GET /api/broadcast/my-rounds`, `GET /api/broadcast/{id}`, `GET /api/broadcast/{tourSlug}/{roundSlug}/{roundId}`, `GET /api/broadcast/round/{roundId}.pgn`, `GET /api/broadcast/{id}.pgn`, `GET /api/stream/broadcast/round/{roundId}.pgn`, `POST /api/broadcast/round/{roundId}/push`, `POST /api/broadcast/round/{roundId}/reset`, `GET /broadcast/{id}/players`, `GET /broadcast/{id}/players/{playerId}`, `GET /broadcast/{id}/teams/standings`, `POST /broadcast/new`, `POST /broadcast/{id}/edit`, `POST /broadcast/{id}/new`, `POST /broadcast/round/{roundId}/edit`

**`client.tv()`** — `litchee::api::broadcasting::tv`  (4 endpoints)

`GET /api/tv/channels`, `GET /api/tv/feed`, `GET /api/tv/{channel}`, `GET /api/tv/{channel}/feed`

</details>

<details>
<summary><b>Database</b> (separate hosts: explorer / tablebase)</summary>

**`client.opening_explorer()`** — `litchee::api::database::opening_explorer`  (4 endpoints, `explorer.lichess.org`)

`GET /masters`, `GET /lichess`, `GET /player`, `GET /masters/pgn/{gameId}`

**`client.tablebase()`** — `litchee::api::database::tablebase`  (3 endpoints, `tablebase.lichess.org`)

`GET /standard`, `GET /atomic`, `GET /antichess`

**`client.analysis()`** — `litchee::api::database::analysis`  (1 endpoint)

`GET /api/cloud-eval`

</details>

<details>
<summary><b>Gameplay</b></summary>

**`client.board()`** — `litchee::api::gameplay::board`  (13 endpoints)

`GET /api/stream/event`, `GET /api/board/game/stream/{gameId}`, `POST /api/board/game/{gameId}/move/{move}`, `POST /api/board/game/{gameId}/abort`, `POST /api/board/game/{gameId}/resign`, `POST /api/board/game/{gameId}/draw/{accept}`, `POST /api/board/game/{gameId}/takeback/{accept}`, `POST /api/board/game/{gameId}/claim-victory`, `POST /api/board/game/{gameId}/claim-draw`, `POST /api/board/game/{gameId}/berserk`, `GET /api/board/game/{gameId}/chat`, `POST /api/board/game/{gameId}/chat`, `POST /api/board/seek`

**`client.bot()`** — `litchee::api::gameplay::bot`  (13 endpoints)

`POST /api/bot/account/upgrade`, `GET /api/bot/online`, `GET /api/stream/event`, `GET /api/bot/game/stream/{gameId}`, `POST /api/bot/game/{gameId}/move/{move}`, `POST /api/bot/game/{gameId}/abort`, `POST /api/bot/game/{gameId}/resign`, `POST /api/bot/game/{gameId}/draw/{accept}`, `POST /api/bot/game/{gameId}/takeback/{accept}`, `POST /api/bot/game/{gameId}/claim-victory`, `POST /api/bot/game/{gameId}/claim-draw`, `GET /api/bot/game/{gameId}/chat`, `POST /api/bot/game/{gameId}/chat`

**`client.challenges()`** — `litchee::api::gameplay::challenges`  (11 endpoints)

`GET /api/challenge`, `GET /api/challenge/{challengeId}/show`, `POST /api/challenge/{username}`, `POST /api/challenge/ai`, `POST /api/challenge/open`, `POST /api/challenge/{challengeId}/accept`, `POST /api/challenge/{challengeId}/decline`, `POST /api/challenge/{challengeId}/cancel`, `POST /api/challenge/{gameId}/start-clocks`, `POST /api/round/{gameId}/add-time/{seconds}`, `POST /api/token/admin-challenge`

**`client.bulk_pairing()`** — `litchee::api::gameplay::bulk_pairing`  (6 endpoints)

`GET /api/bulk-pairing`, `POST /api/bulk-pairing`, `GET /api/bulk-pairing/{id}`, `DELETE /api/bulk-pairing/{id}`, `POST /api/bulk-pairing/{id}/start-clocks`, `GET /api/bulk-pairing/{id}/games`

**`client.games()`** — `litchee::api::gameplay::games`  (13 endpoints)

`GET /game/export/{gameId}`, `GET /api/games/user/{username}`, `POST /api/games/export/_ids`, `GET /api/games/export/bookmarks`, `GET /api/games/export/imports`, `GET /api/account/playing`, `GET /api/user/{username}/current-game`, `GET /game/{gameId}/chat`, `POST /api/import`, `GET /api/stream/game/{id}`, `POST /api/stream/games-by-users`, `POST /api/stream/games/{streamId}`, `POST /api/stream/games/{streamId}/add`

</details>

<details>
<summary><b>Engine</b> (work endpoints on `engine.lichess.ovh`)</summary>

**`client.external_engine()`** — `litchee::api::engine::external_engine`  (8 endpoints)

`GET /api/external-engine`, `POST /api/external-engine`, `GET /api/external-engine/{id}`, `PUT /api/external-engine/{id}`, `DELETE /api/external-engine/{id}`, `POST /api/external-engine/{id}/analyse`, `POST /api/external-engine/work`, `POST /api/external-engine/work/{id}`

</details>

> The OAuth `GET /oauth` endpoint is not a request the client makes — it's the
> URL you redirect the user's browser to. `client.oauth().authorization_url(…)`
> builds it. `GET /api/stream/event` is shared by the Board and Bot APIs.

## Design & technical choices

- **Async-first on `tokio` + `reqwest` (rustls).** Async is required because many
  Lichess endpoints stream newline-delimited JSON (`application/x-ndjson`):
  event streams, board/bot game state, game/tournament exports, TV feeds.
- **Ergonomic streaming.** Streaming endpoints return
  `BoxStream<'static, Result<T>>` — `Unpin`, `Send`, and consumable directly with
  `StreamExt::next()`. Lines are buffered across network chunks and keep-alive
  blanks are skipped.
- **Exhaustive, matchable errors.** Every failure maps to a specific
  [`LichessError`] variant: a structured `ApiError` (status → typed kind + body
  message + `Retry-After`), a typed `OAuthError`, transport/decode/stream
  failures, and PKCE validation errors.
- **Resilient by configuration.** Connect/read timeouts are tunable on the
  builder (the read timeout defaults to 5 minutes so long-lived NDJSON streams
  aren't killed), the NDJSON line buffer is bounded as a DoS guard
  (`max_line_bytes`), and rate-limited (`429`) requests can be retried
  **opt-in** via [`RetryPolicy`] — it waits the response's `Retry-After` when
  present, otherwise exponential backoff clamped to a ceiling. Tokens are held
  in a `Secret` wrapper that redacts them from `Debug` output.
- **Builder pattern.** The client (`LichessClient::builder()`) and every request
  with optional parameters (game export, challenges, tournaments, …) use
  builders rather than wide function signatures.
- **Four hosts, one client.** `lichess.org`, `explorer.lichess.org`,
  `tablebase.lichess.org`, and `engine.lichess.ovh` are routed internally; each
  is overridable on the builder (for self-hosted lila, `localhost`, or mocks).
- **Strong, forward-compatible types.** Every DTO is prefixed `Lichess*` and is
  `#[non_exhaustive]`. Where the API returns very large or evolving aggregates
  (e.g. perf stats, activity feeds, broadcast nested payloads), the documented
  fields are typed and the remainder is preserved losslessly in `serde_json`
  values — nothing is dropped.
- **Organized by business concern.** The module tree mirrors the API's own
  organization, grouped into categories under `src/api/` (see below). Core
  plumbing (`client`, `config`, `error`, `http`, `model`, `stream`) lives at the
  crate root.
- **Tested deterministically.** Each endpoint has an integration test that runs
  against a [`wiremock`](https://docs.rs/wiremock) mock server with fixtures
  derived from the spec's own examples; pure logic (PKCE derivation, NDJSON
  splitting, error mapping, serde round-trips) has unit tests. CI runs
  `fmt`, `clippy -D warnings`, the test suite, the doc build, and an MSRV check.
- **Safety & quality gates.** `#![forbid(unsafe_code)]`, clippy `pedantic`,
  `missing_docs`, and a self-imposed ≤900 LOC/file and ≤20 LOC/method limit.

### Project layout

```
src/
  lib.rs
  client/ config/ error/ http/ model/ stream/   # core plumbing
  api/
    auth/          oauth
    users/         account, players, fide
    social/        relations, messaging, teams
    tournaments/   arena, swiss, simuls
    training/      puzzles, studies
    broadcasting/  broadcasts, tv
    database/      opening_explorer, tablebase, analysis
    gameplay/      board, bot, challenges, bulk_pairing, games
    engine/        external_engine
```

## The vendored OpenAPI spec (`reference/` submodule)

This repository includes the **official Lichess OpenAPI specification** as a git
submodule at [`reference/lichess-api/`](reference/lichess-api) (source:
[lichess-org/api](https://github.com/lichess-org/api)). It is the **source of
truth** for the client and is used during development to:

- **Model DTOs faithfully** — field names, optionality, and enums are taken
  directly from the spec's schemas.
- **Drive deterministic tests** — integration-test fixtures are derived from the
  spec's documented examples, so tests need no network or credentials.
- **Verify coverage** — implemented endpoints are diffed against the spec to
  guarantee full (184/184) coverage as the API evolves.

The submodule is **development-only**: it is excluded from the published crate.
Clone it with:

```bash
git clone --recurse-submodules https://github.com/obazin/litchee
# or, in an existing clone:
git submodule update --init --recursive
```

## Development

```bash
cargo build
cargo test                                       # unit + integration tests
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt
cargo doc --no-deps --open
```

## Contributing

Contributions are welcome. Please keep the project conventions: one concern per
module under the right `src/api/` category, `Lichess*`-prefixed DTOs, an
integration test per endpoint plus unit tests for pure functions, and a clean
`cargo fmt` / `cargo clippy -D warnings`. The vendored spec under `reference/` is
the source of truth for endpoints and types.

## License

Licensed under the [MIT License](LICENSE).

[Lichess API]: https://lichess.org/api
[`LichessError`]: https://docs.rs/litchee/latest/litchee/error/enum.LichessError.html
[`RetryPolicy`]: https://docs.rs/litchee/latest/litchee/struct.RetryPolicy.html
