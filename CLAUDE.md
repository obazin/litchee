# litchee — Project Guide for Claude

`litchee` is an **async, builder-pattern Rust client for the Lichess API** with
first-class **PKCE OAuth** support. The goal is **feature parity** with the
official Lichess API.

The official OpenAPI spec is vendored as a git submodule at
`reference/lichess-api/` (source: <https://github.com/lichess-org/api>). Treat
`reference/lichess-api/doc/specs/lichess-api.yaml` and the `schemas/` + `tags/`
folders beside it as the **source of truth** for endpoints, DTOs, and errors.

---

## Non-negotiable rules

These are hard constraints. Do not violate them; if a task seems to require it,
stop and flag it.

### 1. Folder organization mirrors the API's business concerns
Each API concern is a folder containing a `mod.rs`, grouped into **category**
folders by business concern, all under `src/api/`. The core/plumbing modules
(`client`, `config`, `error`, `http`, `model`, `stream`) stay at `src/` root.

```
src/
  lib.rs
  client/ config/ error/ http/ model/ stream/   # core (not API concerns)
  api/
    auth/          oauth
    users/         account, users, fide
    social/        relations, messaging, teams
    tournaments/   arena, swiss, simuls
    training/      puzzles, studies
    broadcasting/  broadcasts, tv
    database/      opening_explorer, tablebase, analysis
    gameplay/      board, bot, challenges, bulk_pairing, games
    engine/        external_engine
```

Public paths follow the tree, e.g. `litchee::api::gameplay::board::*`. Endpoint
accessors are unaffected (`client.board()`, `client.account()`, …). When adding
a new concern, place it in the most fitting category (create a new category if
none fits).

DTOs belong with the concern they serve (or in the shared root `model` module
when genuinely cross-cutting). Do not create a single god-module of types.

### 2. Size limits (enforced, no exceptions)
- **No file may exceed 600 lines of code.**
- **No method/function may exceed 20 lines of code.**

When approaching a limit, **split eagerly**: extract helpers, break a file into a
folder with `mod.rs` and submodules. Prefer many small, single-purpose units over
large ones.

### 3. Exhaustive, specific error mapping
**Every error the API can return must map to a specific Rust error variant** — not
a generic catch-all. Model error responses faithfully from the spec
(`schemas/*Error*.yaml`, `NotFound.yaml`, `OAuthError.yaml`, HTTP status codes,
rate limiting `429`, etc.). The error type must let a caller match on *what*
went wrong, not just *that* something did.

### 4. DTO naming convention
Every DTO derived from the API is prefixed with `Lichess`
(e.g. `LichessGame`, `LichessUser`, `LichessStudy`, `LichessToken`).

### 5. Testing is mandatory
- **An integration test for EVERY implemented endpoint.** No endpoint is "done"
  until it has one.
- **Unit tests for every pure internal function** (PKCE derivation, NDJSON line
  parsing, query/form serialization, etc.).

### 6. Builder pattern
Public construction (the client, and any request with optional parameters) uses
the builder pattern. Endpoints with many optional query/form params expose a
builder rather than a wide function signature.

---

## Workflow rules

### Commits
- **One atomic commit per independent task.** A commit is one coherent change;
  do not bundle unrelated work.
- **Conventional prefixes** on every commit message: `feat:`, `fix:`,
  `refacto:`, `chore:`, `test:`, `docs:`, etc.
- **Do not add a `Co-Authored-By` trailer** (or any AI-attribution trailer) to
  commit messages.
- Only commit when the change is complete and verified.

### Code review before every commit
- **Every set of changes must pass a code-review agent before it is committed.**
  Run the review, address findings, then commit. Do not commit unreviewed work.

### Navigation & docs
- **Use the LSP (rust-analyzer)** for code navigation, symbol lookup, references,
  and type info whenever possible — prefer it over plain text search for
  understanding Rust code.
- **Consult context7** for Rust crate / language documentation whenever there is
  any doubt about an API, signature, or idiom (reqwest, serde, tokio, futures,
  base64, sha2, etc.). Don't guess — look it up.

---

## Inspiration policy (ideas only — never copy)
Two projects may be consulted **for ideas about structure and ergonomics only**:
- `tontsa28/licheszter` (Rust) — <https://github.com/tontsa28/licheszter>
- `berserk` (Python) — the official-ish Python client.

**Hard rule: never duplicate their content, and deliberately avoid using the same
names** (types, methods, modules). Borrow concepts, not code or identifiers.
`litchee` must be an independent implementation.

---

## Architecture (intended shape)
- **Runtime:** async-first on `tokio` + `reqwest` (rustls). Async is required
  because many Lichess endpoints stream **NDJSON** (`application/x-ndjson`):
  event streams, board game state, game exports.
- **Client:** `LichessClient` built via `LichessClient::builder()`. Holds the
  `reqwest::Client`, base URL, and optional auth token.
- **Auth:** personal access token *and* OAuth2 Authorization Code flow **with
  PKCE** (lives in the `oauth` concern). DTO: `LichessToken`; a `Scope` type
  enumerates every scope from the spec.
- **Streaming:** a shared NDJSON helper turns a byte stream into a
  `Stream<Item = Result<T>>`, splitting on newlines and skipping keep-alive
  blank lines.
- **Endpoints:** each concern exposes an API accessor off the client
  (e.g. `client.account()`, `client.board()`), returning typed `Lichess*` DTOs
  or streams.

---

## Common commands
```bash
cargo build
cargo test                 # unit + integration tests
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt
git submodule update --init --recursive   # fetch the vendored API spec
```

## Definition of done (per endpoint/task)
1. Code respects the size limits and folder organization above.
2. All reachable API errors are mapped to specific error variants.
3. Unit tests cover the pure logic; an integration test covers the endpoint.
4. `cargo clippy -D warnings` and `cargo fmt --check` are clean.
5. The change passed a code-review agent.
6. Committed atomically with a conventional, prefixed message.
