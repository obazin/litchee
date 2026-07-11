//! DTOs for a user's daily activity feed (`GET /api/user/{username}/activity`).
//!
//! The most common categories (games, puzzles, tournaments) are strongly typed;
//! the remaining, more varied categories are preserved verbatim in
//! [`LichessActivity::other`].

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The time range of a [`LichessActivity`] entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessActivityInterval {
    /// Start time (Unix milliseconds).
    pub start: i64,
    /// End time (Unix milliseconds).
    pub end: i64,
}

/// The rating change over an activity score.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessActivityScoreProgress {
    /// Rating before the day's games.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before: Option<i32>,
    /// Rating after the day's games.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<i32>,
}

/// Win/loss/draw counts and rating progress for one category.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessActivityScore {
    /// Games won.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub win: Option<u32>,
    /// Games lost.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loss: Option<u32>,
    /// Games drawn.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub draw: Option<u32>,
    /// The rating change over the day.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rp: Option<LichessActivityScoreProgress>,
}

/// The puzzle activity for a day.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessActivityPuzzles {
    /// The puzzle score.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<LichessActivityScore>,
}

/// A tournament reference in an activity entry.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessActivityTournament {
    /// The tournament id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The tournament name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// One tournament result within a day's activity.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessActivityTournamentResult {
    /// The tournament.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tournament: Option<LichessActivityTournament>,
    /// Number of games played.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nb_games: Option<u32>,
    /// The user's score.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<u32>,
    /// The user's final rank.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rank: Option<u32>,
    /// The rank as a percentile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rank_percent: Option<u32>,
}

/// The tournaments played in a day.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessActivityTournaments {
    /// Number of tournaments entered.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nb: Option<u32>,
    /// The best tournament results.
    #[serde(default)]
    pub best: Vec<LichessActivityTournamentResult>,
}

/// One day of a user's activity. `GET /api/user/{username}/activity`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessActivity {
    /// The time range this entry covers.
    pub interval: LichessActivityInterval,
    /// Games played, keyed by perf (e.g. `blitz`, `bullet`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub games: Option<HashMap<String, LichessActivityScore>>,
    /// Puzzle activity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub puzzles: Option<LichessActivityPuzzles>,
    /// Tournament activity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tournaments: Option<LichessActivityTournaments>,
    /// Any remaining categories (storm, racer, follows, studies, teams, …),
    /// preserved verbatim.
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}
