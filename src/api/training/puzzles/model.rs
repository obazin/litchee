//! DTOs for the Puzzles concern.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::model::{LichessColor, LichessTitle};

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
