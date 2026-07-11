//! The Puzzles API: daily puzzle, lookups, the next puzzle, and activity.
//!
//! Reached through [`LichessClient::puzzles`].

use std::collections::HashMap;

use futures_util::stream::BoxStream;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;
use crate::model::{LichessColor, LichessTitle};

/// Accessor for the Puzzles API.
#[derive(Debug)]
pub struct PuzzlesApi<'a> {
    client: &'a LichessClient,
}

/// Query parameters for the next puzzle.
#[derive(Debug, Default, Serialize)]
struct NextQuery<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    angle: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    difficulty: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<LichessColor>,
}

/// Query parameters for puzzle activity.
#[derive(Debug, Default, Serialize)]
struct ActivityQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    max: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    before: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    since: Option<i64>,
}

impl<'a> PuzzlesApi<'a> {
    /// Binds the accessor to a client.
    pub(crate) fn new(client: &'a LichessClient) -> Self {
        Self { client }
    }

    /// Gets the daily puzzle. `GET /api/puzzle/daily`
    pub async fn daily(&self) -> Result<LichessPuzzleAndGame> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/puzzle/daily");
        http::json(request, "LichessPuzzleAndGame").await
    }

    /// Gets a puzzle by id. `GET /api/puzzle/{id}`
    pub async fn get(&self, id: &str) -> Result<LichessPuzzleAndGame> {
        let path = format!("/api/puzzle/{}", http::segment(id));
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "LichessPuzzleAndGame").await
    }

    /// Gets a new puzzle, optionally filtered by theme, difficulty, and color.
    ///
    /// `GET /api/puzzle/next`
    pub async fn next(
        &self,
        angle: Option<&str>,
        difficulty: Option<&str>,
        color: Option<LichessColor>,
    ) -> Result<LichessPuzzleAndGame> {
        let query = NextQuery {
            angle,
            difficulty,
            color,
        };
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/puzzle/next")
            .query(&query);
        http::json(request, "LichessPuzzleAndGame").await
    }

    /// Streams the authenticated user's puzzle activity, most recent first.
    ///
    /// `before`/`since` bound the window by timestamp (ms). Requires the
    /// `puzzle:read` scope. `GET /api/puzzle/activity`
    pub async fn activity(
        &self,
        max: Option<u32>,
        before: Option<i64>,
        since: Option<i64>,
    ) -> Result<BoxStream<'static, Result<LichessPuzzleActivity>>> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/puzzle/activity")
            .query(&ActivityQuery { max, before, since });
        http::stream(request, self.client.max_line_bytes()).await
    }

    /// Gets a batch of puzzles for the given angle (theme/opening).
    ///
    /// `difficulty` (e.g. `easiest`…`hardest`) and `color` (`white`/`black`)
    /// tune the selection. `GET /api/puzzle/batch/{angle}`
    pub async fn batch(
        &self,
        angle: &str,
        nb: u32,
        difficulty: Option<&str>,
        color: Option<LichessColor>,
    ) -> Result<LichessPuzzleBatch> {
        let path = format!("/api/puzzle/batch/{}", http::segment(angle));
        let request = self
            .client
            .request(Method::GET, Host::Default, &path)
            .query(&[("nb", nb)])
            .query(&[("difficulty", difficulty)])
            .query(&[("color", color)]);
        http::json(request, "LichessPuzzleBatch").await
    }

    /// Submits puzzle solutions and updates ratings.
    ///
    /// When `nb` > 0, the response also carries a fresh batch of `nb` puzzles
    /// (equivalent to calling [`batch`](Self::batch)); pass `0` to skip it.
    /// `POST /api/puzzle/batch/{angle}`
    pub async fn solve_batch(
        &self,
        angle: &str,
        solutions: &[LichessPuzzleSolution],
        nb: u32,
    ) -> Result<LichessPuzzleSolveResponse> {
        let path = format!("/api/puzzle/batch/{}", http::segment(angle));
        let body = serde_json::json!({ "solutions": solutions });
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .query(&[("nb", nb)])
            .json(&body);
        http::json(request, "LichessPuzzleSolveResponse").await
    }

    /// Gets the authenticated user's puzzle dashboard for the last `days` days.
    ///
    /// `GET /api/puzzle/dashboard/{days}`
    pub async fn dashboard(&self, days: u32) -> Result<LichessPuzzleDashboard> {
        // `days` is numeric, so it needs no percent-encoding (see `http::segment`).
        let path = format!("/api/puzzle/dashboard/{days}");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "LichessPuzzleDashboard").await
    }

    /// Gets a puzzle replay session for a theme over the last `days` days.
    ///
    /// `GET /api/puzzle/replay/{days}/{theme}`
    pub async fn replay(&self, days: u32, theme: &str) -> Result<LichessPuzzleReplay> {
        let path = format!("/api/puzzle/replay/{days}/{}", http::segment(theme));
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "LichessPuzzleReplay").await
    }

    /// Gets a user's Puzzle Storm dashboard.
    ///
    /// `days` sets how many days of history to include. `GET /api/storm/dashboard/{username}`
    pub async fn storm_dashboard(
        &self,
        username: &str,
        days: Option<u32>,
    ) -> Result<LichessStormDashboard> {
        let path = format!("/api/storm/dashboard/{}", http::segment(username));
        let request = self
            .client
            .request(Method::GET, Host::Default, &path)
            .query(&[("days", days)]);
        http::json(request, "LichessStormDashboard").await
    }

    /// Gets a puzzle race by id. `GET /api/racer/{id}`
    pub async fn racer(&self, id: &str) -> Result<LichessPuzzleRacer> {
        let path = format!("/api/racer/{}", http::segment(id));
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "LichessPuzzleRacer").await
    }

    /// Creates a new puzzle race. `POST /api/racer`
    pub async fn create_racer(&self) -> Result<LichessPuzzleRacer> {
        let request = self
            .client
            .request(Method::POST, Host::Default, "/api/racer");
        http::json(request, "LichessPuzzleRacer").await
    }
}

impl LichessClient {
    /// Puzzles API: the daily puzzle, lookups, and activity.
    #[must_use]
    pub fn puzzles(&self) -> PuzzlesApi<'_> {
        PuzzlesApi::new(self)
    }
}

/// Display info for a puzzle game's perf.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessPuzzlePerf {
    /// The perf key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// The perf name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// A player in the game a puzzle is drawn from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessPuzzlePlayer {
    /// The player's color.
    pub color: LichessColor,
    /// The player's id.
    pub id: String,
    /// The player's name.
    pub name: String,
    /// The player's rating.
    pub rating: u32,
    /// The player's title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<LichessTitle>,
    /// The player's flair.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flair: Option<String>,
    /// The chosen Patron wing color.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patron_color: Option<u8>,
}

/// The game a puzzle is drawn from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessPuzzleGame {
    /// The game id.
    pub id: String,
    /// The clock, as a human-readable string.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clock: Option<String>,
    /// Perf display info.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub perf: Option<LichessPuzzlePerf>,
    /// The game PGN.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pgn: Option<String>,
    /// The two players.
    #[serde(default)]
    pub players: Vec<LichessPuzzlePlayer>,
    /// Whether the game was rated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rated: Option<bool>,
}

/// A puzzle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessPuzzle {
    /// The puzzle id.
    pub id: String,
    /// The ply the puzzle starts from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_ply: Option<u32>,
    /// The number of times the puzzle has been played.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plays: Option<u32>,
    /// The puzzle's rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rating: Option<u32>,
    /// The starting position (FEN).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fen: Option<String>,
    /// The last move before the puzzle, in UCI.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_move: Option<String>,
    /// The solution moves, in UCI.
    #[serde(default)]
    pub solution: Vec<String>,
    /// The puzzle's themes.
    #[serde(default)]
    pub themes: Vec<String>,
}

/// A puzzle together with the game it is drawn from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessPuzzleAndGame {
    /// The source game.
    pub game: LichessPuzzleGame,
    /// The puzzle.
    pub puzzle: LichessPuzzle,
}

/// One entry in a user's puzzle activity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessPuzzleActivity {
    /// When the puzzle was solved (Unix milliseconds).
    pub date: i64,
    /// The puzzle.
    pub puzzle: LichessPuzzle,
    /// Whether it was solved correctly.
    pub win: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn activity_query_omits_unset_fields() {
        let query = ActivityQuery {
            max: Some(10),
            before: None,
            since: Some(5),
        };
        assert_eq!(
            serde_urlencoded::to_string(&query).unwrap(),
            "max=10&since=5"
        );
    }

    #[test]
    fn parses_puzzle_and_game() {
        let json = r#"{"game":{"id":"g","clock":"3+0",
            "players":[{"color":"white","id":"a","name":"A","rating":1500},
                       {"color":"black","id":"b","name":"B","rating":1490}],"rated":true},
            "puzzle":{"id":"p","initialPly":20,"plays":100,"rating":1600,
                      "solution":["e2e4"],"themes":["mateIn1"]}}"#;
        let pg: LichessPuzzleAndGame = serde_json::from_str(json).unwrap();
        assert_eq!(pg.puzzle.id, "p");
        assert_eq!(pg.puzzle.solution, vec!["e2e4"]);
        assert_eq!(pg.game.players.len(), 2);
    }

    #[test]
    fn parses_activity_without_initial_ply() {
        let json = r#"{"date":1623000000000,"win":true,
            "puzzle":{"id":"p","fen":"x","lastMove":"e2e4","plays":1,"rating":1500,
                      "solution":["a1a2"],"themes":["short"]}}"#;
        let activity: LichessPuzzleActivity = serde_json::from_str(json).unwrap();
        assert!(activity.win);
        assert_eq!(activity.puzzle.initial_ply, None);
    }
}

/// A batch of puzzles. `GET`/`POST /api/puzzle/batch/{angle}`
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessPuzzleBatch {
    /// The puzzles in the batch.
    #[serde(default)]
    pub puzzles: Vec<LichessPuzzleAndGame>,
    /// The user's puzzle Glicko rating, if present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub glicko: Option<Value>,
}

/// One solved-puzzle round in a batch-solve response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessPuzzleRound {
    /// The puzzle id.
    pub id: String,
    /// Whether it was solved correctly.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub win: Option<bool>,
    /// The rating change resulting from the attempt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rating_diff: Option<i32>,
}

/// The response to submitting a batch of solutions. `POST /api/puzzle/batch/{angle}`.
///
/// Distinct from [`LichessPuzzleBatch`] (the `GET` select response): it adds the
/// solved `rounds`, and `puzzles` is only populated when `nb` > 0 was requested.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessPuzzleSolveResponse {
    /// The fresh puzzle batch, present only when `nb` > 0 was requested.
    #[serde(default)]
    pub puzzles: Vec<LichessPuzzleAndGame>,
    /// The user's puzzle Glicko rating, if present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub glicko: Option<Value>,
    /// The submitted solutions with their outcomes.
    #[serde(default)]
    pub rounds: Vec<LichessPuzzleRound>,
}

/// A solution submitted for a puzzle in a batch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct LichessPuzzleSolution {
    /// The puzzle id.
    pub id: String,
    /// Whether it was solved correctly.
    pub win: bool,
    /// Whether the attempt was rated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rated: Option<bool>,
}

impl LichessPuzzleSolution {
    /// Creates a solution for a puzzle.
    #[must_use]
    pub fn new(id: impl Into<String>, win: bool) -> Self {
        Self {
            id: id.into(),
            win,
            rated: None,
        }
    }
}

/// The puzzle dashboard. `GET /api/puzzle/dashboard/{days}`
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessPuzzleDashboard {
    /// The number of days covered.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub days: Option<u32>,
    /// Aggregate results across all themes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub global: Option<Value>,
    /// Per-theme results.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub themes: Option<Value>,
}

/// The Puzzle Storm dashboard. `GET /api/storm/dashboard/{username}`
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessStormDashboard {
    /// High scores (all-time/day/week/month).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub high: Option<Value>,
    /// Per-day statistics.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub days: Option<Value>,
}

/// A puzzle replay session. `GET /api/puzzle/replay/{days}/{theme}`
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessPuzzleReplay {
    /// The theme/opening angle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub angle: Option<Value>,
    /// The replay progress.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replay: Option<Value>,
}

/// A puzzle race. `GET /api/racer/{id}`, `POST /api/racer`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessPuzzleRacer {
    /// The race id.
    pub id: String,
    /// The race URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Any other fields.
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}
