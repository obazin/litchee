//! # litchee
//!
//! An asynchronous, builder-pattern Rust client for the [Lichess API].
//!
//! `litchee` targets feature parity with the official API. It is async-first
//! (built on `tokio` + `reqwest`), because many Lichess endpoints stream
//! newline-delimited JSON (`application/x-ndjson`) — event streams, board and
//! bot game state, game exports, and more.
//!
//! ## Authentication
//!
//! Two flows are supported:
//!
//! - **Personal access token** — pass a token to the client builder.
//! - **`OAuth2` Authorization Code flow with PKCE** — so applications can
//!   "Log in with Lichess". See the `oauth` module.
//!
//! ## Naming
//!
//! Every data-transfer object decoded from the API is prefixed with `Lichess`
//! (for example `LichessUser`, `LichessGame`, `LichessToken`).
//!
//! ## Quick start
//!
//! Build a client and call an endpoint group accessor:
//!
//! ```no_run
//! use litchee::LichessClient;
//!
//! # async fn run() -> litchee::Result<()> {
//! let client = LichessClient::builder().token("lip_your_token").build()?;
//!
//! // A simple JSON request.
//! let me = client.account().profile().await?;
//! println!("Logged in as {}", me.user.username);
//!
//! // A streaming (NDJSON) request.
//! use futures_util::StreamExt;
//! let mut games = client.games().export_user("bobby").max(5).stream().await?;
//! while let Some(game) = games.next().await {
//!     println!("game {}", game?.id);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! See the `examples/` directory for runnable programs, and the `oauth` module
//! for the "Log in with Lichess" PKCE flow.
//!
//! [Lichess API]: https://lichess.org/api

pub mod api;
pub mod error;
pub mod model;

mod client;
mod config;
mod http;
mod secret;
mod stream;

pub use client::{LichessClient, LichessClientBuilder};
pub use error::{LichessError, Result};
pub use secret::Secret;
