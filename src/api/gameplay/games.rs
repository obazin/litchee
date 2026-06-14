//! The Games API: export games (JSON, NDJSON, or PGN) and import PGN.
//!
//! Reached through [`LichessClient::games`].

use futures_util::stream::BoxStream;
use reqwest::Method;
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;
use crate::model::{LichessColor, LichessLightUser, LichessSpeed, LichessVariantKey};

/// The `application/x-ndjson` content type.
const NDJSON: &str = "application/x-ndjson";

/// Accessor for the Games API.
#[derive(Debug)]
pub struct GamesApi<'a> {
    client: &'a LichessClient,
}

impl<'a> GamesApi<'a> {
    /// Binds the accessor to a client.
    pub(crate) fn new(client: &'a LichessClient) -> Self {
        Self { client }
    }

    /// Starts a single-game export. `GET /game/export/{gameId}`
    ///
    /// Finish with [`json`](GameExportRequest::json) or
    /// [`pgn`](GameExportRequest::pgn).
    #[must_use]
    pub fn export(&self, game_id: &'a str) -> GameExportRequest<'a> {
        GameExportRequest::new(self.client, game_id)
    }

    /// Starts a user's games export. `GET /api/games/user/{username}`
    ///
    /// Finish with [`stream`](UserGamesRequest::stream) or
    /// [`pgn`](UserGamesRequest::pgn).
    #[must_use]
    pub fn export_user(&self, username: &'a str) -> UserGamesRequest<'a> {
        UserGamesRequest::new(self.client, username)
    }

    /// Gets the game the user is currently playing, if any.
    ///
    /// `GET /api/user/{username}/current-game`
    pub async fn current_game(&self, username: &str) -> Result<LichessGame> {
        let path = format!("/api/user/{}/current-game", http::segment(username));
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "LichessGame").await
    }

    /// Imports a game from its PGN.
    ///
    /// Requires authentication. `POST /api/import`
    pub async fn import_game(&self, pgn: &str) -> Result<LichessImportedGame> {
        let request = self
            .client
            .request(Method::POST, Host::Default, "/api/import")
            .form(&[("pgn", pgn)]);
        http::json(request, "LichessImportedGame").await
    }

    /// Gets the games the authenticated user is currently playing.
    ///
    /// `GET /api/account/playing`
    pub async fn now_playing(&self) -> Result<LichessNowPlaying> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/account/playing");
        http::json(request, "LichessNowPlaying").await
    }

    /// Gets the spectator chat of a game. `GET /game/{gameId}/chat`
    pub async fn chat(&self, game_id: &str) -> Result<Vec<LichessGameChatMessage>> {
        let path = format!("/game/{}/chat", http::segment(game_id));
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "Vec<LichessGameChatMessage>").await
    }

    /// Exports several games by id (NDJSON). `POST /api/games/export/_ids`
    pub async fn export_by_ids(
        &self,
        ids: &[&str],
    ) -> Result<BoxStream<'static, Result<LichessGame>>> {
        let request = self
            .client
            .request(Method::POST, Host::Default, "/api/games/export/_ids")
            .header(ACCEPT, NDJSON)
            .header(CONTENT_TYPE, "text/plain")
            .body(ids.join(","));
        http::stream(request, self.client.max_line_bytes()).await
    }

    /// Streams the authenticated user's bookmarked games (NDJSON).
    /// `GET /api/games/export/bookmarks`
    pub async fn export_bookmarks(&self) -> Result<BoxStream<'static, Result<LichessGame>>> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/games/export/bookmarks")
            .header(ACCEPT, NDJSON);
        http::stream(request, self.client.max_line_bytes()).await
    }

    /// Streams the authenticated user's imported games (NDJSON).
    /// `GET /api/games/export/imports`
    pub async fn export_imports(&self) -> Result<BoxStream<'static, Result<LichessGame>>> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/games/export/imports")
            .header(ACCEPT, NDJSON);
        http::stream(request, self.client.max_line_bytes()).await
    }

    /// Streams a game's moves as they are played. `GET /api/stream/game/{id}`
    pub async fn stream_moves(
        &self,
        game_id: &str,
    ) -> Result<BoxStream<'static, Result<LichessGameMoveUpdate>>> {
        let path = format!("/api/stream/game/{}", http::segment(game_id));
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::stream(request, self.client.max_line_bytes()).await
    }

    /// Streams games played by the given users as they start/finish (NDJSON).
    /// `POST /api/stream/games-by-users`
    pub async fn stream_by_users(
        &self,
        usernames: &[&str],
    ) -> Result<BoxStream<'static, Result<LichessGame>>> {
        let request = self
            .client
            .request(Method::POST, Host::Default, "/api/stream/games-by-users")
            .header(CONTENT_TYPE, "text/plain")
            .body(usernames.join(","));
        http::stream(request, self.client.max_line_bytes()).await
    }

    /// Streams a custom set of games by id (NDJSON).
    /// `POST /api/stream/games/{streamId}`
    pub async fn stream_by_ids(
        &self,
        stream_id: &str,
        ids: &[&str],
    ) -> Result<BoxStream<'static, Result<LichessGame>>> {
        let path = format!("/api/stream/games/{}", http::segment(stream_id));
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .header(CONTENT_TYPE, "text/plain")
            .body(ids.join(","));
        http::stream(request, self.client.max_line_bytes()).await
    }

    /// Adds game ids to an existing game stream.
    /// `POST /api/stream/games/{streamId}/add`
    pub async fn add_to_stream(&self, stream_id: &str, ids: &[&str]) -> Result<()> {
        let path = format!("/api/stream/games/{}/add", http::segment(stream_id));
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .header(CONTENT_TYPE, "text/plain")
            .body(ids.join(","));
        http::ok(request).await
    }
}

impl LichessClient {
    /// Games API: export and import games.
    #[must_use]
    pub fn games(&self) -> GamesApi<'_> {
        GamesApi::new(self)
    }
}

/// The `application/x-chess-pgn` content type.
const PGN: &str = "application/x-chess-pgn";

/// Builder for exporting a single game (`GET /game/export/{gameId}`).
#[derive(Debug)]
pub struct GameExportRequest<'a> {
    client: &'a LichessClient,
    game_id: &'a str,
    query: SingleExportQuery,
}

/// Query options shared by both export endpoints' formatting.
#[derive(Debug, Default, Serialize)]
struct SingleExportQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    moves: Option<bool>,
    #[serde(rename = "pgnInJson", skip_serializing_if = "Option::is_none")]
    pgn_in_json: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    clocks: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    evals: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    accuracy: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    opening: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    literate: Option<bool>,
}

impl<'a> GameExportRequest<'a> {
    /// Creates the request builder.
    pub(crate) fn new(client: &'a LichessClient, game_id: &'a str) -> Self {
        Self {
            client,
            game_id,
            query: SingleExportQuery::default(),
        }
    }

    /// Includes the full PGN inside the JSON response (`pgn` field).
    #[must_use]
    pub fn pgn_in_json(mut self, value: bool) -> Self {
        self.query.pgn_in_json = Some(value);
        self
    }

    /// Includes clock comments / fields.
    #[must_use]
    pub fn clocks(mut self, value: bool) -> Self {
        self.query.clocks = Some(value);
        self
    }

    /// Includes analysis evaluations.
    #[must_use]
    pub fn evals(mut self, value: bool) -> Self {
        self.query.evals = Some(value);
        self
    }

    /// Includes the opening name.
    #[must_use]
    pub fn opening(mut self, value: bool) -> Self {
        self.query.opening = Some(value);
        self
    }

    /// Includes per-player accuracy (JSON only).
    #[must_use]
    pub fn accuracy(mut self, value: bool) -> Self {
        self.query.accuracy = Some(value);
        self
    }

    /// Executes the export, returning the game as JSON.
    pub async fn json(self) -> Result<LichessGame> {
        let path = format!("/game/export/{}", http::segment(self.game_id));
        let request = self
            .client
            .request(Method::GET, Host::Default, &path)
            .query(&self.query);
        http::json(request, "LichessGame").await
    }

    /// Executes the export, returning the game as a PGN string.
    pub async fn pgn(self) -> Result<String> {
        let path = format!("/game/export/{}", http::segment(self.game_id));
        let request = self
            .client
            .request(Method::GET, Host::Default, &path)
            .header(ACCEPT, PGN)
            .query(&self.query);
        http::text(request).await
    }
}

/// Builder for exporting a user's games (`GET /api/games/user/{username}`).
#[derive(Debug)]
pub struct UserGamesRequest<'a> {
    client: &'a LichessClient,
    username: &'a str,
    query: UserGamesQuery<'a>,
}

/// Query parameters for exporting a user's games.
#[derive(Debug, Default, Serialize)]
struct UserGamesQuery<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    max: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    since: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    until: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vs: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rated: Option<bool>,
    #[serde(rename = "perfType", skip_serializing_if = "Option::is_none")]
    perf_type: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    analysed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ongoing: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    finished: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    opening: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    evals: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    clocks: Option<bool>,
}

impl<'a> UserGamesRequest<'a> {
    /// Creates the request builder.
    pub(crate) fn new(client: &'a LichessClient, username: &'a str) -> Self {
        Self {
            client,
            username,
            query: UserGamesQuery::default(),
        }
    }

    /// Limits the number of games downloaded.
    #[must_use]
    pub fn max(mut self, count: u32) -> Self {
        self.query.max = Some(count);
        self
    }

    /// Only games played since this timestamp (Unix milliseconds).
    #[must_use]
    pub fn since(mut self, timestamp: i64) -> Self {
        self.query.since = Some(timestamp);
        self
    }

    /// Only games played until this timestamp (Unix milliseconds).
    #[must_use]
    pub fn until(mut self, timestamp: i64) -> Self {
        self.query.until = Some(timestamp);
        self
    }

    /// Only rated (`true`) or casual (`false`) games.
    #[must_use]
    pub fn rated(mut self, rated: bool) -> Self {
        self.query.rated = Some(rated);
        self
    }

    /// Only games in these speeds/variants (comma-separated perf types).
    #[must_use]
    pub fn perf_type(mut self, perf_type: &'a str) -> Self {
        self.query.perf_type = Some(perf_type);
        self
    }

    /// Only games played as this color (`"white"` or `"black"`).
    #[must_use]
    pub fn color(mut self, color: &'a str) -> Self {
        self.query.color = Some(color);
        self
    }

    /// Only currently-ongoing games.
    #[must_use]
    pub fn ongoing(mut self, ongoing: bool) -> Self {
        self.query.ongoing = Some(ongoing);
        self
    }

    /// Executes the export, streaming games as decoded JSON values.
    pub async fn stream(self) -> Result<BoxStream<'static, Result<LichessGame>>> {
        let path = format!("/api/games/user/{}", http::segment(self.username));
        let request = self
            .client
            .request(Method::GET, Host::Default, &path)
            .header(ACCEPT, NDJSON)
            .query(&self.query);
        http::stream(request, self.client.max_line_bytes()).await
    }

    /// Executes the export, returning all games as one PGN string.
    pub async fn pgn(self) -> Result<String> {
        let path = format!("/api/games/user/{}", http::segment(self.username));
        let request = self
            .client
            .request(Method::GET, Host::Default, &path)
            .header(ACCEPT, PGN)
            .query(&self.query);
        http::text(request).await
    }
}

/// The terminal (or current) status of a game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum LichessGameStatusName {
    /// The game has been created but not started.
    Created,
    /// The game is in progress.
    Started,
    /// The game was aborted.
    Aborted,
    /// Won by checkmate.
    Mate,
    /// A player resigned.
    Resign,
    /// Drawn by stalemate.
    Stalemate,
    /// A player's time ran out (flagged).
    Timeout,
    /// Drawn.
    Draw,
    /// A player ran out of time.
    Outoftime,
    /// A player was caught cheating.
    Cheat,
    /// A player did not start in time.
    NoStart,
    /// The game ended in an unknown way.
    UnknownFinish,
    /// A draw was claimed by insufficient material.
    InsufficientMaterialClaim,
    /// The game ended by a variant-specific rule.
    VariantEnd,
}

/// The opening of a game.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessGameOpening {
    /// The ECO code (e.g. `B01`).
    pub eco: String,
    /// The opening name.
    pub name: String,
    /// The ply at which the opening is identified.
    pub ply: u32,
}

/// A judgment annotation on an analysed move.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessMoveJudgment {
    /// The severity (`Inaccuracy`, `Mistake`, or `Blunder`).
    pub name: String,
    /// The human-readable comment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

/// Per-move engine analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessGameMoveAnalysis {
    /// Evaluation in centipawns.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eval: Option<i32>,
    /// Moves until forced mate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mate: Option<i32>,
    /// Best move in UCI notation (only if the played move was inaccurate).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub best: Option<String>,
    /// Best variation in SAN notation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variation: Option<String>,
    /// Judgment annotation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub judgment: Option<LichessMoveJudgment>,
}

/// Aggregate analysis statistics for one player in a game.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessPlayerAnalysis {
    /// Number of inaccuracies.
    pub inaccuracy: u32,
    /// Number of mistakes.
    pub mistake: u32,
    /// Number of blunders.
    pub blunder: u32,
    /// Average centipawn loss.
    pub acpl: u32,
    /// Accuracy percentage, when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accuracy: Option<u32>,
}

/// One side of a game.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessGamePlayer {
    /// The player's light user info (absent for AI players).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<LichessLightUser>,
    /// The player's rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rating: Option<u32>,
    /// The rating change from this game.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rating_diff: Option<i32>,
    /// The player's name (for anonymous or AI players).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Whether the rating is provisional.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provisional: Option<bool>,
    /// The AI level, for games against the computer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ai_level: Option<u32>,
    /// Aggregate analysis for this player.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub analysis: Option<LichessPlayerAnalysis>,
    /// The player's team id, in team games.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team: Option<String>,
}

/// Both sides of a game.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessGamePlayers {
    /// The white player.
    pub white: LichessGamePlayer,
    /// The black player.
    pub black: LichessGamePlayer,
}

/// A game's clock configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessGameClock {
    /// Initial time in seconds.
    pub initial: u32,
    /// Increment per move in seconds.
    pub increment: u32,
    /// Total estimated time in seconds.
    pub total_time: u32,
}

/// The ply boundaries of a game's phases.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessGameDivision {
    /// Ply at which the middlegame begins.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub middle: Option<u32>,
    /// Ply at which the endgame begins.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<u32>,
}

/// A game, as returned in JSON by the export and stream endpoints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessGame {
    /// The game id.
    pub id: String,
    /// Whether the game is rated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rated: Option<bool>,
    /// The variant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<LichessVariantKey>,
    /// The speed category.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed: Option<LichessSpeed>,
    /// The perf key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub perf: Option<String>,
    /// Creation time (Unix milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    /// Last-move time (Unix milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_move_at: Option<i64>,
    /// The game status.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<LichessGameStatusName>,
    /// The game source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// The players.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub players: Option<LichessGamePlayers>,
    /// The initial FEN, for games not started from the standard position.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_fen: Option<String>,
    /// The winner's color, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub winner: Option<LichessColor>,
    /// The opening.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opening: Option<LichessGameOpening>,
    /// The moves in UCI or SAN, space-separated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub moves: Option<String>,
    /// The full PGN, when requested with `pgnInJson`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pgn: Option<String>,
    /// Days per turn, for correspondence games.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub days_per_turn: Option<u32>,
    /// Per-move analysis, when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub analysis: Option<Vec<LichessGameMoveAnalysis>>,
    /// The arena tournament id, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tournament: Option<String>,
    /// The swiss tournament id, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub swiss: Option<String>,
    /// The clock configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clock: Option<LichessGameClock>,
    /// Per-move clock readings, in centiseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clocks: Option<Vec<u32>>,
    /// The phase boundaries.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub division: Option<LichessGameDivision>,
}

/// The result of importing a game.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessImportedGame {
    /// The id of the imported game.
    pub id: String,
    /// The URL of the imported game.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_game() {
        let json = r#"{
            "id":"q7ZvsdUF","rated":true,"variant":"standard","speed":"blitz",
            "perf":"blitz","createdAt":1514505150384,"status":"mate",
            "winner":"white",
            "players":{
                "white":{"user":{"id":"a","name":"A"},"rating":1600,"ratingDiff":8},
                "black":{"user":{"id":"b","name":"B"},"rating":1590,"ratingDiff":-8}
            },
            "opening":{"eco":"B01","name":"Scandinavian","ply":2},
            "moves":"e4 d5 exd5",
            "clock":{"initial":300,"increment":3,"totalTime":420}
        }"#;
        let game: LichessGame = serde_json::from_str(json).unwrap();
        assert_eq!(game.id, "q7ZvsdUF");
        assert_eq!(game.status, Some(LichessGameStatusName::Mate));
        assert_eq!(game.winner, Some(LichessColor::White));
        assert_eq!(game.players.unwrap().white.rating_diff, Some(8));
        assert_eq!(game.clock.unwrap().total_time, 420);
    }

    #[test]
    fn parses_minimal_game() {
        let game: LichessGame = serde_json::from_str(r#"{"id":"abcd1234"}"#).unwrap();
        assert_eq!(game.id, "abcd1234");
        assert!(game.players.is_none());
    }

    #[test]
    fn status_uses_camel_case_names() {
        let game: LichessGame =
            serde_json::from_str(r#"{"id":"x","status":"variantEnd"}"#).unwrap();
        assert_eq!(game.status, Some(LichessGameStatusName::VariantEnd));
    }
}

/// The opponent in a [`LichessNowPlayingGame`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessNowPlayingOpponent {
    /// The opponent's id.
    pub id: String,
    /// The opponent's username.
    pub username: String,
    /// The opponent's rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rating: Option<u32>,
    /// The opponent's rating change.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rating_diff: Option<i32>,
    /// The AI level, if playing the computer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ai: Option<u32>,
}

/// A game the authenticated user is currently playing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessNowPlayingGame {
    /// The game id.
    pub game_id: String,
    /// The full game id (includes the player token).
    pub full_id: String,
    /// The authenticated user's color.
    pub color: LichessColor,
    /// The current position (FEN).
    pub fen: String,
    /// Whether the user has moved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_moved: Option<bool>,
    /// Whether it is the user's turn.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_my_turn: Option<bool>,
    /// The last move in UCI.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_move: Option<String>,
    /// The opponent.
    pub opponent: LichessNowPlayingOpponent,
    /// The perf key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub perf: Option<String>,
    /// Whether the game is rated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rated: Option<bool>,
    /// Seconds left on the user's clock.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seconds_left: Option<i64>,
    /// The game source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// The speed category.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed: Option<LichessSpeed>,
    /// The variant key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<LichessVariantKey>,
    /// The arena tournament id, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tournament_id: Option<String>,
    /// The swiss tournament id, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub swiss_id: Option<String>,
}

/// The games the authenticated user is currently playing.
/// `GET /api/account/playing`
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessNowPlaying {
    /// The ongoing games.
    #[serde(default)]
    pub now_playing: Vec<LichessNowPlayingGame>,
}

/// A spectator chat message on a game.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessGameChatMessage {
    /// The sender's username.
    pub user: String,
    /// The message text.
    pub text: String,
}

/// A move-by-move update from a game move stream. `GET /api/stream/game/{id}`
///
/// The first message is the full game; later messages carry the latest move
/// and clocks. Typed common fields plus a lossless `other` map.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessGameMoveUpdate {
    /// The current position (FEN).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fen: Option<String>,
    /// The last move in UCI.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lm: Option<String>,
    /// White's clock in centiseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wc: Option<i64>,
    /// Black's clock in centiseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bc: Option<i64>,
    /// Any other fields (e.g. the full first message).
    #[serde(flatten)]
    pub other: std::collections::HashMap<String, serde_json::Value>,
}
