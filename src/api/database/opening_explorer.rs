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

/// A game mode the player explorer can be filtered to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExplorerMode {
    /// Casual (unrated) games.
    Casual,
    /// Rated games.
    Rated,
}

impl ExplorerMode {
    /// The wire representation, `"casual"` or `"rated"`.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            ExplorerMode::Casual => "casual",
            ExplorerMode::Rated => "rated",
        }
    }
}

/// Comma-joins string parts into the wire form, or `None` when there are none
/// (an empty filter means "no restriction", so the param is omitted). Borrows
/// the parts, so `&'static str` enum encodings are joined without a per-element
/// allocation.
fn join_csv<'s>(parts: impl Iterator<Item = &'s str>) -> Option<String> {
    let joined = parts.collect::<Vec<_>>().join(",");
    (!joined.is_empty()).then_some(joined)
}

/// Serializable `/masters` query. `since`/`until` are calendar years.
#[derive(Debug, Default, Serialize)]
struct MastersQuery<'a> {
    fen: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    play: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    since: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    until: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    moves: Option<u32>,
    #[serde(rename = "topGames", skip_serializing_if = "Option::is_none")]
    top_games: Option<u32>,
}

/// Serializable `/lichess` query. `speeds`/`ratings` are pre-joined.
#[derive(Debug, Default, Serialize)]
struct LichessQuery<'a> {
    fen: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    play: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    variant: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    speeds: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ratings: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    since: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    until: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    moves: Option<u32>,
    #[serde(rename = "topGames", skip_serializing_if = "Option::is_none")]
    top_games: Option<u32>,
    #[serde(rename = "recentGames", skip_serializing_if = "Option::is_none")]
    recent_games: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    history: Option<bool>,
}

/// Serializable `/player` query. `speeds`/`modes` are pre-joined.
#[derive(Debug, Default, Serialize)]
struct PlayerQuery<'a> {
    player: &'a str,
    color: LichessColor,
    #[serde(skip_serializing_if = "Option::is_none")]
    variant: Option<&'a str>,
    fen: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    play: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    speeds: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    modes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    since: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    until: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    moves: Option<u32>,
    #[serde(rename = "recentGames", skip_serializing_if = "Option::is_none")]
    recent_games: Option<u32>,
}

/// Builder for a masters-database lookup. Finish with [`send`](Self::send).
#[derive(Debug)]
pub struct MastersExplorerRequest<'a> {
    client: &'a LichessClient,
    query: MastersQuery<'a>,
}

impl<'a> MastersExplorerRequest<'a> {
    /// Creates the request builder for the position `fen`.
    pub(crate) fn new(client: &'a LichessClient, fen: &'a str) -> Self {
        Self {
            client,
            query: MastersQuery {
                fen,
                ..Default::default()
            },
        }
    }

    /// Comma-separated UCI moves to apply to `fen` before the lookup.
    #[must_use]
    pub fn play(mut self, play: &'a str) -> Self {
        self.query.play = Some(play);
        self
    }

    /// Only games from this year or later.
    #[must_use]
    pub fn since(mut self, year: u16) -> Self {
        self.query.since = Some(year);
        self
    }

    /// Only games from this year or earlier.
    #[must_use]
    pub fn until(mut self, year: u16) -> Self {
        self.query.until = Some(year);
        self
    }

    /// Number of most-common moves to return.
    #[must_use]
    pub fn moves(mut self, moves: u32) -> Self {
        self.query.moves = Some(moves);
        self
    }

    /// Number of top games to return (max 15).
    #[must_use]
    pub fn top_games(mut self, count: u32) -> Self {
        self.query.top_games = Some(count);
        self
    }

    /// Executes the lookup. `GET /masters`
    pub async fn send(self) -> Result<LichessExplorerResult> {
        let request = self
            .client
            .request(Method::GET, Host::OpeningExplorer, "/masters")
            .query(&self.query);
        http::json(request, "LichessExplorerResult").await
    }
}

/// Builder for a Lichess games lookup. Finish with [`send`](Self::send).
#[derive(Debug)]
pub struct LichessExplorerRequest<'a> {
    client: &'a LichessClient,
    query: LichessQuery<'a>,
}

impl<'a> LichessExplorerRequest<'a> {
    /// Creates the request builder for the position `fen`.
    pub(crate) fn new(client: &'a LichessClient, fen: &'a str) -> Self {
        Self {
            client,
            query: LichessQuery {
                fen,
                ..Default::default()
            },
        }
    }

    /// Comma-separated UCI moves to apply to `fen` before the lookup.
    #[must_use]
    pub fn play(mut self, play: &'a str) -> Self {
        self.query.play = Some(play);
        self
    }

    /// Chess variant (defaults to standard when unset).
    #[must_use]
    pub fn variant(mut self, variant: &'a str) -> Self {
        self.query.variant = Some(variant);
        self
    }

    /// Restrict to these game speeds (empty = all).
    #[must_use]
    pub fn speeds(mut self, speeds: &[LichessSpeed]) -> Self {
        self.query.speeds = join_csv(speeds.iter().map(|s| s.as_str()));
        self
    }

    /// Restrict to these rating bands (empty = all).
    #[must_use]
    pub fn ratings(mut self, ratings: &[RatingGroup]) -> Self {
        let values: Vec<String> = ratings.iter().map(|r| r.as_u16().to_string()).collect();
        self.query.ratings = join_csv(values.iter().map(String::as_str));
        self
    }

    /// Only games from this `YYYY-MM` or later.
    #[must_use]
    pub fn since(mut self, since: &'a str) -> Self {
        self.query.since = Some(since);
        self
    }

    /// Only games from this `YYYY-MM` or earlier.
    #[must_use]
    pub fn until(mut self, until: &'a str) -> Self {
        self.query.until = Some(until);
        self
    }

    /// Number of most-common moves to return.
    #[must_use]
    pub fn moves(mut self, moves: u32) -> Self {
        self.query.moves = Some(moves);
        self
    }

    /// Number of top games to return (max 4).
    #[must_use]
    pub fn top_games(mut self, count: u32) -> Self {
        self.query.top_games = Some(count);
        self
    }

    /// Number of recent games to return (max 4).
    #[must_use]
    pub fn recent_games(mut self, count: u32) -> Self {
        self.query.recent_games = Some(count);
        self
    }

    /// Whether to include the month-by-month history.
    #[must_use]
    pub fn history(mut self, history: bool) -> Self {
        self.query.history = Some(history);
        self
    }

    /// Executes the lookup. `GET /lichess`
    pub async fn send(self) -> Result<LichessExplorerResult> {
        let request = self
            .client
            .request(Method::GET, Host::OpeningExplorer, "/lichess")
            .query(&self.query);
        http::json(request, "LichessExplorerResult").await
    }
}

/// Builder for a player games lookup. Finish with [`stream`](Self::stream).
#[derive(Debug)]
pub struct PlayerExplorerRequest<'a> {
    client: &'a LichessClient,
    query: PlayerQuery<'a>,
}

impl<'a> PlayerExplorerRequest<'a> {
    /// Creates the request builder for `player` on `color` at position `fen`.
    pub(crate) fn new(
        client: &'a LichessClient,
        player: &'a str,
        color: LichessColor,
        fen: &'a str,
    ) -> Self {
        Self {
            client,
            query: PlayerQuery {
                player,
                color,
                fen,
                ..Default::default()
            },
        }
    }

    /// Comma-separated UCI moves to apply to `fen` before the lookup.
    #[must_use]
    pub fn play(mut self, play: &'a str) -> Self {
        self.query.play = Some(play);
        self
    }

    /// Chess variant (defaults to standard when unset).
    #[must_use]
    pub fn variant(mut self, variant: &'a str) -> Self {
        self.query.variant = Some(variant);
        self
    }

    /// Restrict to these game speeds (empty = all).
    #[must_use]
    pub fn speeds(mut self, speeds: &[LichessSpeed]) -> Self {
        self.query.speeds = join_csv(speeds.iter().map(|s| s.as_str()));
        self
    }

    /// Restrict to these game modes (empty = all).
    #[must_use]
    pub fn modes(mut self, modes: &[ExplorerMode]) -> Self {
        self.query.modes = join_csv(modes.iter().map(|m| m.as_str()));
        self
    }

    /// Only games from this `YYYY-MM` or later.
    #[must_use]
    pub fn since(mut self, since: &'a str) -> Self {
        self.query.since = Some(since);
        self
    }

    /// Only games from this `YYYY-MM` or earlier.
    #[must_use]
    pub fn until(mut self, until: &'a str) -> Self {
        self.query.until = Some(until);
        self
    }

    /// Number of most-common moves to return.
    #[must_use]
    pub fn moves(mut self, moves: u32) -> Self {
        self.query.moves = Some(moves);
        self
    }

    /// Number of recent games to return (max 8).
    #[must_use]
    pub fn recent_games(mut self, count: u32) -> Self {
        self.query.recent_games = Some(count);
        self
    }

    /// Executes the lookup, streaming results as they are computed (NDJSON).
    /// `GET /player`
    pub async fn stream(self) -> Result<BoxStream<'static, Result<LichessExplorerResult>>> {
        let request = self
            .client
            .request(Method::GET, Host::OpeningExplorer, "/player")
            .query(&self.query);
        http::stream(request, self.client.max_line_bytes()).await
    }
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
    /// Returns a builder; refine it (date range, counts) and finish with
    /// [`MastersExplorerRequest::send`].
    #[must_use]
    pub fn masters(&self, fen: &'a str) -> MastersExplorerRequest<'a> {
        MastersExplorerRequest::new(self.client, fen)
    }

    /// Looks up a position in the Lichess games database. `GET /lichess`
    ///
    /// Returns a builder; refine it (speeds, ratings, date range, …) and finish
    /// with [`LichessExplorerRequest::send`].
    #[must_use]
    pub fn lichess(&self, fen: &'a str) -> LichessExplorerRequest<'a> {
        LichessExplorerRequest::new(self.client, fen)
    }

    /// Streams a player's opening statistics for a position. `GET /player`
    ///
    /// Returns a builder; refine it (variant, speeds, modes, …) and finish with
    /// [`PlayerExplorerRequest::stream`].
    #[must_use]
    pub fn player(
        &self,
        player: &'a str,
        color: LichessColor,
        fen: &'a str,
    ) -> PlayerExplorerRequest<'a> {
        PlayerExplorerRequest::new(self.client, player, color, fen)
    }

    /// Downloads a master game as PGN. `GET /masters/pgn/{gameId}`
    pub async fn masters_pgn(&self, game_id: &str) -> Result<String> {
        let path = format!("/masters/pgn/{}", http::segment(game_id));
        let request = self
            .client
            .request(Method::GET, Host::OpeningExplorer, &path);
        http::text(request).await
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

    /// A client with no config — enough to build request builders offline.
    fn client() -> LichessClient {
        LichessClient::builder().build().expect("client builds")
    }

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
    fn join_csv_omits_empty_and_comma_joins() {
        assert_eq!(join_csv(std::iter::empty()), None);
        assert_eq!(
            join_csv(["a", "b", "c"].into_iter()),
            Some("a,b,c".to_owned())
        );
    }

    #[test]
    fn lichess_default_query_is_fen_only() {
        let client = client();
        let req = client.opening_explorer().lichess("thefen");
        assert_eq!(req.query.fen, "thefen");
        assert!(req.query.play.is_none() && req.query.speeds.is_none());
    }

    #[test]
    fn lichess_speeds_and_ratings_are_comma_joined() {
        let client = client();
        let req = client
            .opening_explorer()
            .lichess("f")
            .speeds(&[LichessSpeed::Blitz, LichessSpeed::Rapid])
            .ratings(&[RatingGroup::R1600, RatingGroup::R1800]);
        assert_eq!(req.query.speeds.as_deref(), Some("blitz,rapid"));
        assert_eq!(req.query.ratings.as_deref(), Some("1600,1800"));
    }

    #[test]
    fn empty_filters_are_omitted_but_play_is_kept() {
        let client = client();
        let req = client
            .opening_explorer()
            .lichess("f")
            .play("e2e4")
            .speeds(&[])
            .ratings(&[]);
        assert_eq!(req.query.play, Some("e2e4"));
        assert!(req.query.speeds.is_none() && req.query.ratings.is_none());
    }

    #[test]
    fn masters_query_uses_year_bounds() {
        let client = client();
        let req = client
            .opening_explorer()
            .masters("f")
            .since(1952)
            .until(2020)
            .top_games(15);
        assert_eq!(req.query.since, Some(1952));
        assert_eq!(req.query.until, Some(2020));
        assert_eq!(req.query.top_games, Some(15));
    }

    #[test]
    fn player_query_joins_speeds_and_modes() {
        let client = client();
        let req = client
            .opening_explorer()
            .player("bobby", LichessColor::Black, "f")
            .speeds(&[LichessSpeed::Bullet])
            .modes(&[ExplorerMode::Rated, ExplorerMode::Casual])
            .recent_games(5);
        assert_eq!(req.query.player, "bobby");
        assert_eq!(req.query.color, LichessColor::Black);
        assert_eq!(req.query.speeds.as_deref(), Some("bullet"));
        assert_eq!(req.query.modes.as_deref(), Some("rated,casual"));
        assert_eq!(req.query.recent_games, Some(5));
    }

    #[test]
    fn rating_group_round_trips() {
        for group in RatingGroup::ALL {
            assert_eq!(RatingGroup::from_u16(group.as_u16()), Some(group));
        }
        assert_eq!(RatingGroup::from_u16(1234), None);
    }

    #[test]
    fn explorer_mode_wire_values() {
        assert_eq!(ExplorerMode::Casual.as_str(), "casual");
        assert_eq!(ExplorerMode::Rated.as_str(), "rated");
    }
}
