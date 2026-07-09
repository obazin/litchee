# Changelog

All notable changes to **litchee** are documented in this file. It is generated
from [Conventional Commits](https://www.conventionalcommits.org) by
[git-cliff](https://git-cliff.org) — edit commit messages, not this file.
## [v0.1.4](https://github.com/obazin/litchee/releases/tag/v0.1.4) — 2026-07-09
Changes since [v0.1.3](https://github.com/obazin/litchee/releases/tag/v0.1.3).


### Features
- **broadcasts:** Tiebreaks (array) and grouping (nested) tour fields
- Bulk-pairing and study create form fields
- **challenges:** Remaining accept/cancel/create/open params
- POST form fields for tournament joins and board seek
- **broadcasts:** List/filter params for official/top/by_user/my_rounds
- **teams:** List/filter params for members, requests, and tournaments
- List/filter params for users, account, and puzzles
- List/filter query params for tournaments, tablebase, bot, games
- Add PgnExportOptions to study and broadcast PGN exports
- **broadcasts:** Full create/update form parity for tours and rounds
- **swiss:** Entry conditions + full create/edit form parity
- **arena:** Entry conditions + full create/update form parity
- **tv:** Full parameter parity for tvChannelGames
- **bulk-pairing:** Full parameter parity for bulkPairingIdGamesGet
- **swiss:** Full parameter parity for gamesBySwiss
- **arena:** Full parameter parity for gamesByTournament
- **games:** Full game-export parameter parity via GameExportOptions
- **model:** Add GameExportOptions cross-cutting export params


### Fixes
- **challenges:** Send Accept: x-ndjson on the keepAliveStream challenge
- **challenges:** KeepAliveStream returns an NDJSON stream, not one JSON


### Refactor
- **broadcasts:** Split broadcasts.rs into an oauth-style folder
- **http:** Consolidate game-export Accept media types into shared consts
- **games:** Split games.rs into an oauth-style folder


### Documentation
- Point broadcast tournament PGN export to streaming endpoints
- Fix stale README broadcasts().official() example
## [v0.1.3](https://github.com/obazin/litchee/releases/tag/v0.1.3) — 2026-07-08
Changes since [v0.1.2](https://github.com/obazin/litchee/releases/tag/v0.1.2).


### Features
- **opening-explorer:** Complete masters + player parameter parity via builders
- **opening-explorer:** Filter the Lichess games query by speed + rating
## [v0.1.2](https://github.com/obazin/litchee/releases/tag/v0.1.2) — 2026-07-02
Changes since [v0.1.1](https://github.com/obazin/litchee/releases/tag/v0.1.1).


### Fixes
- Bump anyhow to 1.0.103 to resolve RUSTSEC-2026-0190 (#18)
## [v0.1.1](https://github.com/obazin/litchee/releases/tag/v0.1.1) — 2026-06-27
Changes since [v0.1.0](https://github.com/obazin/litchee/releases/tag/v0.1.0).


### Features
- **games:** Add nbMyTurn to account/playing response


### Fixes
- **ci:** Run the stable job on stable, not the toml-pinned MSRV
- Stop Dependabot from bumping dtolnay/rust-toolchain (our MSRV pin)
## [v0.1.0](https://github.com/obazin/litchee/releases/tag/v0.1.0) — 2026-06-15


### Features
- Add opt-in retry of rate-limited (429) requests
- Make the NDJSON line cap configurable; document it as a DoS guard
- Add a read timeout to detect stalled connections
- Add HEAD /api/study/{id}.pgn study metadata endpoint


### Fixes
- Raise default read timeout to 5 minutes
- Bound the NDJSON line buffer to avoid unbounded growth
- Map SwissUnauthorisedEdit 401 to a specific error variant
- Redact team entry password from Debug output
- Percent-encode user-supplied URL path segments
- Redact external-engine secrets from Debug output
- Redact bearer token from client Debug output


### Performance
- Return a Display adapter from http::segment to avoid an allocation


### Refactor
- Redact secrets via a Secret newtype instead of per-struct Debug
- Flatten each API concern into a single module file


### Documentation
- Record cargo-public-api nightly requirement in the API-freeze playbook
- Refresh README and PKCE guide for the OAuth example and recent features
- Add end-to-end PKCE OAuth example
- Add API-freeze playbook toward 1.0 (todo.md)
- Note why numeric path params skip percent-encoding
- Warn that a non-TLS base URL transmits the token in cleartext
- Add beginner PKCE flow guide and link it from the README
- Point repository URL at obazin/litchee

