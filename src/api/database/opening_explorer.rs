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
use crate::model::{LichessColor, LichessSpeed};

/// Query parameters shared by the masters and Lichess explorers.
#[derive(Debug, Serialize)]
struct ExplorerQuery<'a> {
    fen: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    play: Option<&'a str>,
}

/// A player-rating band the Lichess games explorer can be filtered to. Each
/// value is the inclusive lower bound of a 200-point band (`R0` = below 1000,
/// `R2500` = 2500 and above).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RatingGroup {
    /// Below 1000.
    R0,
    /// 1000–1199.
    R1000,
    /// 1200–1399.
    R1200,
    /// 1400–1599.
    R1400,
    /// 1600–1799.
    R1600,
    /// 1800–1999.
    R1800,
    /// 2000–2199.
    R2000,
    /// 2200–2499.
    R2200,
    /// 2500 and above.
    R2500,
}

impl RatingGroup {
    /// Every rating band, in ascending order — the full set the API accepts.
    pub const ALL: [RatingGroup; 9] = [
        RatingGroup::R0,
        RatingGroup::R1000,
        RatingGroup::R1200,
        RatingGroup::R1400,
        RatingGroup::R1600,
        RatingGroup::R1800,
        RatingGroup::R2000,
        RatingGroup::R2200,
        RatingGroup::R2500,
    ];

    /// The wire value (the band's lower bound), e.g. `1600`.
    #[must_use]
    pub fn as_u16(self) -> u16 {
        match self {
            RatingGroup::R0 => 0,
            RatingGroup::R1000 => 1000,
            RatingGroup::R1200 => 1200,
            RatingGroup::R1400 => 1400,
            RatingGroup::R1600 => 1600,
            RatingGroup::R1800 => 1800,
            RatingGroup::R2000 => 2000,
            RatingGroup::R2200 => 2200,
            RatingGroup::R2500 => 2500,
        }
    }

    /// Parses a band from its wire value, or `None` if it isn't a valid bucket.
    #[must_use]
    pub fn from_u16(value: u16) -> Option<Self> {
        Self::ALL.into_iter().find(|group| group.as_u16() == value)
    }
}

/// Filter parameters for the Lichess games explorer ([`OpeningExplorerApi::lichess`]).
///
/// All fields are optional; [`Default`] is the unfiltered query (every speed and
/// rating band). An empty `speeds`/`ratings` slice means "no restriction".
#[derive(Debug, Clone, Default)]
pub struct LichessExplorerParams<'a> {
    /// Comma-separated UCI moves to apply to `fen` before the lookup.
    pub play: Option<&'a str>,
    /// Chess variant (defaults to standard when unset).
    pub variant: Option<&'a str>,
    /// Restrict to these game speeds (empty = all).
    pub speeds: &'a [LichessSpeed],
    /// Restrict to these rating bands (empty = all).
    pub ratings: &'a [RatingGroup],
    /// Only games from this `YYYY-MM` or later.
    pub since: Option<&'a str>,
    /// Only games from this `YYYY-MM` or earlier.
    pub until: Option<&'a str>,
    /// Number of most-common moves to return.
    pub moves: Option<u32>,
    /// Number of top games to return.
    pub top_games: Option<u32>,
    /// Number of recent games to return.
    pub recent_games: Option<u32>,
    /// Whether to include the month-by-month history.
    pub history: Option<bool>,
}

/// Builds the `/lichess` query string from `fen` + `params`. Empty / unset
/// fields are omitted. Extracted from [`OpeningExplorerApi::lichess`] so it can
/// be unit-tested without a client.
fn build_lichess_query(fen: &str, params: &LichessExplorerParams<'_>) -> Vec<(&'static str, String)> {
    let mut query: Vec<(&'static str, String)> = vec![("fen", fen.to_owned())];
    if let Some(play) = params.play {
        query.push(("play", play.to_owned()));
    }
    if let Some(variant) = params.variant {
        query.push(("variant", variant.to_owned()));
    }
    if !params.speeds.is_empty() {
        let speeds = params
            .speeds
            .iter()
            .map(|speed| speed.as_str())
            .collect::<Vec<_>>()
            .join(",");
        query.push(("speeds", speeds));
    }
    if !params.ratings.is_empty() {
        let ratings = params
            .ratings
            .iter()
            .map(|rating| rating.as_u16().to_string())
            .collect::<Vec<_>>()
            .join(",");
        query.push(("ratings", ratings));
    }
    if let Some(since) = params.since {
        query.push(("since", since.to_owned()));
    }
    if let Some(until) = params.until {
        query.push(("until", until.to_owned()));
    }
    if let Some(moves) = params.moves {
        query.push(("moves", moves.to_string()));
    }
    if let Some(top_games) = params.top_games {
        query.push(("topGames", top_games.to_string()));
    }
    if let Some(recent_games) = params.recent_games {
        query.push(("recentGames", recent_games.to_string()));
    }
    if let Some(history) = params.history {
        query.push(("history", history.to_string()));
    }
    query
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
    ///
    /// `params` filters the query by speed, rating band, date range, etc.;
    /// [`LichessExplorerParams::default`] is the unfiltered lookup.
    pub async fn lichess(
        &self,
        fen: &str,
        params: &LichessExplorerParams<'_>,
    ) -> Result<LichessExplorerResult> {
        let query = build_lichess_query(fen, params);
        let request = self
            .client
            .request(Method::GET, Host::OpeningExplorer, "/lichess")
            .query(&query);
        http::json(request, "LichessExplorerResult").await
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

    #[test]
    fn default_params_query_is_fen_only() {
        let query = build_lichess_query("thefen", &LichessExplorerParams::default());
        assert_eq!(query, vec![("fen", "thefen".to_owned())]);
    }

    #[test]
    fn speeds_and_ratings_are_comma_joined() {
        let params = LichessExplorerParams {
            speeds: &[LichessSpeed::Blitz, LichessSpeed::Rapid],
            ratings: &[RatingGroup::R1600, RatingGroup::R1800],
            ..Default::default()
        };
        let query = build_lichess_query("f", &params);
        assert!(query.contains(&("speeds", "blitz,rapid".to_owned())));
        assert!(query.contains(&("ratings", "1600,1800".to_owned())));
    }

    #[test]
    fn empty_speeds_and_ratings_are_omitted() {
        let params = LichessExplorerParams {
            play: Some("e2e4"),
            ..Default::default()
        };
        let query = build_lichess_query("f", &params);
        assert!(query.iter().all(|(k, _)| *k != "speeds" && *k != "ratings"));
        assert!(query.contains(&("play", "e2e4".to_owned())));
    }

    #[test]
    fn rating_group_round_trips() {
        for group in RatingGroup::ALL {
            assert_eq!(RatingGroup::from_u16(group.as_u16()), Some(group));
        }
        assert_eq!(RatingGroup::from_u16(1234), None);
    }
}
