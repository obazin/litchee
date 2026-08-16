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
Each API concern is a **single module file** (`<concern>.rs`), grouped into
**category** folders by business concern, all under `src/api/`. A category is a
folder with a `mod.rs` that declares its concern files. The core/plumbing modules
(`client`, `config`, `error`, `http`, `model`, `stream`) stay at `src/` root.

```
src/
  lib.rs
  client/ config/ error/ http/ model/ stream/   # core (not API concerns)
  api/
    auth/          oauth/ (a folder — see exception below)
    users/         account.rs, players.rs, fide.rs
    social/        relations.rs, messaging.rs, teams.rs
    tournaments/   arena.rs, swiss.rs, simuls.rs
    training/      puzzles.rs, studies.rs
    broadcasting/  broadcasts.rs, tv.rs
    database/      opening_explorer.rs, tablebase.rs, analysis.rs
    gameplay/      board.rs, bot.rs, challenges.rs, bulk_pairing.rs, games.rs
    engine/        external_engine.rs
```

A concern is **one flat file**: its endpoint accessor/impl, its `Lichess*` DTOs,
and its tests all live together in `<concern>.rs` — do not split a concern into a
`mod.rs` + `model.rs` + sub-files folder. **Exception:** `oauth` stays a folder
(`auth/oauth/` with `mod.rs`, `pkce.rs`, `scope.rs`, `token.rs`) because its parts
are genuinely independent units.

Public paths follow the tree, e.g. `litchee::api::gameplay::board::*`. Endpoint
accessors are unaffected (`client.board()`, `client.account()`, …). When adding
a new concern, place it in the most fitting category as a single `<concern>.rs`
file (create a new category folder if none fits).

DTOs belong with the concern they serve (or in the shared root `model` module
when genuinely cross-cutting). Do not create a single god-module of types.

### 2. Size limits (enforced, no exceptions within their boundary)

Two caps, each with its own scope. The boundary is part of the rule: a cap is binding everywhere inside it and silent outside it, so neither has to be argued case by case.

| Cap | Applies to | Does not apply to |
| --- | --- | --- |
| **No file may exceed 900 lines.** | Every `.rs` file git knows about — tracked or newly created — across `src/`, `tests/`, and `examples/` alike. | Gitignored files and the vendored `reference/lichess-api` submodule, which the gate never enumerates. |
| **No method/function may exceed 20 lines of code.** | Everything under `src/`, including the `#[cfg(test)]` unit-test modules that rule 1 keeps inside each concern file. | `tests/` and `examples/`. |

The two caps count differently, so don't read one number against the other: the file cap counts **physical lines**, comments and blanks included (it is about how much you have to scroll through), while the function cap counts **body lines only**, ignoring comments and blanks (it is about how much logic sits in one unit).

The file cap is 900 (not 600) because a concern is a single flat file bundling its endpoints, DTOs, and tests (see rule 1). When approaching the limit, **split eagerly**: extract helpers, or — if a concern genuinely outgrows one file — give it the `oauth`-style folder treatment. Prefer many small, single-purpose units over large ones.

The function cap stops at `src/` because it exists to keep *library* logic decomposable, and that reasoning does not carry over. An integration test is a linear arrange/act/assert narrative and an example is a walkthrough; both are read top-to-bottom, and chopping them into helpers to satisfy a counter makes them harder to follow, not easier. Length in those trees is a judgement call, not a violation.

**How this is enforced.** The file cap runs as `scripts/check-file-size.sh` in CI (no clippy lint covers file length). The function cap rides on `clippy::too_many_lines`, whose threshold `clippy.toml` pins to 20. Run both gates the way CI does with `just check-size`.

That lint is configured crate-wide, so the `src/`-only boundary is not something clippy can express: every file in `tests/` and `examples/` carries a file-level `#![allow(clippy::too_many_lines)]` to mark itself as outside it, and **a new test or example file must carry that allow too**. Forgetting it is caught by `tests/size_boundary_guard.rs`, which names the file and the attribute to add — otherwise the omission would surface much later as a `too_many_lines` error reporting a violation of a rule that does not apply there.

One known limitation, documented in `clippy.toml`: the lint measures a function's *source* span, so a body generated by a `macro_rules!` is measured at the macro definition rather than the expansion, and a macro wrapper sidesteps the cap. Do not use one to dodge the rule.

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
The Rust toolchain comes from the Nix dev shell in `flake.nix`, which consumes the shared [`obazin/rust-projects`](https://github.com/obazin/rust-projects) flake. The pin (1.95.0 — this crate's MSRV) lives there, so bumping Rust moves every project at once; the shell materializes the canonical `rust-toolchain.toml` at the project root as a symlink into the Nix store, which is why the repo carries no copy of its own. Run commands inside it:
```bash
nix develop --command cargo build          # or: direnv allow, then plain cargo
cargo test                 # unit + integration tests
cargo nextest run          # same suite via the nextest runner (also in the shell)
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt
git submodule update --init --recursive   # fetch the vendored API spec
nix flake check            # fmt + clippy + the suite, on the pinned toolchain
```

## Definition of done (per endpoint/task)
1. Code respects the size limits and folder organization above.
2. All reachable API errors are mapped to specific error variants.
3. Unit tests cover the pure logic; an integration test covers the endpoint.
4. `cargo clippy -D warnings` and `cargo fmt --check` are clean.
5. The change passed a code-review agent.
6. Committed atomically with a conventional, prefixed message.
