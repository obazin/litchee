//! The Opening Explorer API: aggregate statistics for positions.
//!
//! Served from `explorer.lichess.org`. Reached through
//! [`LichessClient::opening_explorer`].

use futures_util::stream::BoxStream;
use reqwest::Method;
use serde::Serialize;

use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;

mod model;

pub use model::{
    LichessExplorerGame, LichessExplorerGamePlayer, LichessExplorerMove, LichessExplorerOpening,
    LichessExplorerResult,
};

/// Query parameters shared by the masters and Lichess explorers.
#[derive(Debug, Serialize)]
struct ExplorerQuery<'a> {
    fen: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    play: Option<&'a str>,
}

/// Query parameters for the player explorer.
#[derive(Debug, Serialize)]
struct PlayerQuery<'a> {
    player: &'a str,
    color: &'a str,
    fen: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    play: Option<&'a str>,
}

/// Accessor for the Opening Explorer API.
#[derive(Debug)]
pub struct OpeningExplorerApi<'a> {
    client: &'a LichessClient,
}

impl<'a> OpeningExplorerApi<'a> {
    /// Binds the accessor to a client.
    pub(crate) fn new(client: &'a LichessClient) -> Self {
        Self { client }
    }

    /// Looks up a position in the masters database. `GET /masters`
    ///
    /// `play` is an optional comma-separated list of UCI moves to apply to the
    /// position given by `fen`.
    pub async fn masters(&self, fen: &str, play: Option<&str>) -> Result<LichessExplorerResult> {
        self.lookup("/masters", fen, play).await
    }

    /// Looks up a position in the Lichess games database. `GET /lichess`
    pub async fn lichess(&self, fen: &str, play: Option<&str>) -> Result<LichessExplorerResult> {
        self.lookup("/lichess", fen, play).await
    }

    /// Streams a player's opening statistics for a position (NDJSON).
    ///
    /// The result is sent incrementally as it is computed. `GET /player`
    pub async fn player(
        &self,
        player: &str,
        color: &str,
        fen: &str,
        play: Option<&str>,
    ) -> Result<BoxStream<'static, Result<LichessExplorerResult>>> {
        let query = PlayerQuery {
            player,
            color,
            fen,
            play,
        };
        let request = self
            .client
            .request(Method::GET, Host::OpeningExplorer, "/player")
            .query(&query);
        http::stream(request).await
    }

    /// Downloads a master game as PGN. `GET /masters/pgn/{gameId}`
    pub async fn masters_pgn(&self, game_id: &str) -> Result<String> {
        let path = format!("/masters/pgn/{game_id}");
        let request = self
            .client
            .request(Method::GET, Host::OpeningExplorer, &path);
        http::text(request).await
    }

    /// Issues an explorer lookup against the explorer host.
    async fn lookup(
        &self,
        path: &str,
        fen: &str,
        play: Option<&str>,
    ) -> Result<LichessExplorerResult> {
        let request = self
            .client
            .request(Method::GET, Host::OpeningExplorer, path)
            .query(&ExplorerQuery { fen, play });
        http::json(request, "LichessExplorerResult").await
    }
}

impl LichessClient {
    /// Opening Explorer API (`explorer.lichess.org`).
    #[must_use]
    pub fn opening_explorer(&self) -> OpeningExplorerApi<'_> {
        OpeningExplorerApi::new(self)
    }
}
