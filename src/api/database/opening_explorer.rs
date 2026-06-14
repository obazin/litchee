//! The Opening Explorer API: aggregate statistics for positions.
//!
//! Served from `explorer.lichess.org`. Reached through
//! [`LichessClient::opening_explorer`].

use futures_util::stream::BoxStream;
use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;
use crate::model::LichessColor;

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
        http::stream(request, self.client.max_line_bytes()).await
    }

    /// Downloads a master game as PGN. `GET /masters/pgn/{gameId}`
    pub async fn masters_pgn(&self, game_id: &str) -> Result<String> {
        let path = format!("/masters/pgn/{}", http::segment(game_id));
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

/// The opening identified for a position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessExplorerOpening {
    /// The ECO code.
    pub eco: String,
    /// The opening name.
    pub name: String,
}

/// A player in an opening-explorer game reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessExplorerGamePlayer {
    /// The player's name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The player's rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rating: Option<u32>,
}

/// A reference to a game in the opening explorer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessExplorerGame {
    /// The game id.
    pub id: String,
    /// The move leading to this game (in top/recent game lists).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uci: Option<String>,
    /// The winner, if any.
    #[serde(default)]
    pub winner: Option<LichessColor>,
    /// The white player.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub white: Option<LichessExplorerGamePlayer>,
    /// The black player.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub black: Option<LichessExplorerGamePlayer>,
    /// The year the game was played.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub year: Option<u32>,
    /// The month the game was played.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub month: Option<String>,
}

/// A candidate move with its aggregate statistics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessExplorerMove {
    /// The move in UCI notation.
    pub uci: String,
    /// The move in SAN notation.
    pub san: String,
    /// Number of games White won.
    pub white: u64,
    /// Number of drawn games.
    pub draws: u64,
    /// Number of games Black won.
    pub black: u64,
    /// The average rating of games with this move.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub average_rating: Option<u32>,
    /// A sample game with this move.
    #[serde(default)]
    pub game: Option<LichessExplorerGame>,
}

/// The opening-explorer result for a position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessExplorerResult {
    /// The identified opening, if any.
    #[serde(default)]
    pub opening: Option<LichessExplorerOpening>,
    /// Number of games White won from this position.
    pub white: u64,
    /// Number of drawn games.
    pub draws: u64,
    /// Number of games Black won.
    pub black: u64,
    /// Candidate moves, most popular first.
    #[serde(default)]
    pub moves: Vec<LichessExplorerMove>,
    /// Notable games from this position.
    #[serde(default)]
    pub top_games: Vec<LichessExplorerGame>,
    /// Recent games from this position (Lichess/player databases).
    #[serde(default)]
    pub recent_games: Vec<LichessExplorerGame>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_explorer_result() {
        let json = r#"{"opening":{"eco":"B01","name":"Scandinavian"},
            "white":100,"draws":40,"black":60,
            "moves":[{"uci":"e2e4","san":"e4","white":50,"draws":20,"black":30,
                      "averageRating":2400}],
            "topGames":[{"id":"g","uci":"e2e4","winner":"white",
                         "white":{"name":"A","rating":2700},"year":2020}]}"#;
        let result: LichessExplorerResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.white, 100);
        assert_eq!(result.moves[0].average_rating, Some(2400));
        assert_eq!(result.top_games[0].winner, Some(LichessColor::White));
    }
}
