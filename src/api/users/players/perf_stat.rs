//! DTOs for a user's per-perf statistics (`GET /api/user/{username}/perf/{perf}`).
//!
//! Models the full `PerfStat` schema, including the deeply-nested `stat`
//! aggregate (counts, best/worst results, and result/play streaks).

use serde::{Deserialize, Serialize};

use super::LichessGlicko;
use crate::model::LichessLightUser;

/// The rating part of a [`LichessPerfStat`].
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessPerfStatPerf {
    /// The Glicko-2 rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub glicko: Option<LichessGlicko>,
    /// Number of games played.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nb: Option<u32>,
    /// Recent progress.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progress: Option<i32>,
}

/// A rating extreme (highest/lowest) with the game it occurred in.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessPerfStatRatingAt {
    /// The rating value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub int: Option<i32>,
    /// When it was reached (ISO 8601).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub at: Option<String>,
    /// The game in which it was reached.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game_id: Option<String>,
}

/// A single notable result (a best win or a worst loss).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessPerfStatResult {
    /// The opponent's rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub op_rating: Option<i32>,
    /// The opponent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub op_id: Option<LichessLightUser>,
    /// When the game was played (ISO 8601).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub at: Option<String>,
    /// The game id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game_id: Option<String>,
}

/// A wrapper around a list of notable results.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessPerfStatResults {
    /// The notable results.
    #[serde(default)]
    pub results: Vec<LichessPerfStatResult>,
}

/// Aggregate game counts for a perf.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessPerfStatCount {
    /// Total games.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub all: Option<u32>,
    /// Rated games.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rated: Option<u32>,
    /// Wins.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub win: Option<u32>,
    /// Losses.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loss: Option<u32>,
    /// Draws.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub draw: Option<u32>,
    /// Tournament games.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tour: Option<u32>,
    /// Berserked games.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub berserk: Option<u32>,
    /// Average opponent rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub op_avg: Option<f64>,
    /// Total time spent, in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seconds: Option<u64>,
    /// Number of disconnects.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disconnects: Option<u32>,
}

/// The game boundary (start or end) of a streak span.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessPerfStatStreakStamp {
    /// When the boundary game was played (ISO 8601).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub at: Option<String>,
    /// The boundary game id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game_id: Option<String>,
}

/// One streak span: its length and the games that bound it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessPerfStatStreakSpan {
    /// The span length (games, or seconds for time streaks).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub v: Option<u64>,
    /// The first game of the span.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<LichessPerfStatStreakStamp>,
    /// The last game of the span.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<LichessPerfStatStreakStamp>,
}

/// A current and maximum streak span.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessPerfStatStreak {
    /// The current span.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cur: Option<LichessPerfStatStreakSpan>,
    /// The longest span.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<LichessPerfStatStreakSpan>,
}

/// Win and loss streaks by result.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessPerfStatResultStreak {
    /// The winning streaks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub win: Option<LichessPerfStatStreak>,
    /// The losing streaks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loss: Option<LichessPerfStatStreak>,
}

/// Streaks by number of games and by time played.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessPerfStatPlayStreak {
    /// Streaks measured in games.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nb: Option<LichessPerfStatStreak>,
    /// Streaks measured in time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time: Option<LichessPerfStatStreak>,
    /// When the user last played this perf (ISO 8601).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_date: Option<String>,
}

/// The detailed `stat` aggregate of a [`LichessPerfStat`].
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessPerfStatDetails {
    /// The highest rating ever reached.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub highest: Option<LichessPerfStatRatingAt>,
    /// The lowest rating ever reached.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lowest: Option<LichessPerfStatRatingAt>,
    /// The best wins by opponent rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub best_wins: Option<LichessPerfStatResults>,
    /// The worst losses by opponent rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worst_losses: Option<LichessPerfStatResults>,
    /// Aggregate game counts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count: Option<LichessPerfStatCount>,
    /// Win/loss streaks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_streak: Option<LichessPerfStatResultStreak>,
    /// Games/time streaks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub play_streak: Option<LichessPerfStatPlayStreak>,
}

/// Statistics for one of a user's perfs. `GET /api/user/{username}/perf/{perf}`
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessPerfStat {
    /// The user's percentile within this perf.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub percentile: Option<f64>,
    /// The user's rank within this perf.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rank: Option<u32>,
    /// The user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<LichessLightUser>,
    /// The rating details.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub perf: Option<LichessPerfStatPerf>,
    /// The detailed statistics aggregate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stat: Option<LichessPerfStatDetails>,
}
