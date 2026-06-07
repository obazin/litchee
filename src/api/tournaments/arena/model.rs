//! DTOs for the Arena Tournaments concern.

use serde::{Deserialize, Serialize};

use crate::model::{LichessLightUser, LichessTitle};

/// An arena clock (`limit` + `increment`, in seconds).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessArenaClock {
    /// Initial time in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Increment per move in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub increment: Option<u32>,
}

/// Perf display info for an arena.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessArenaPerf {
    /// The perf key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// The perf name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The perf icon.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

/// A summary of an arena tournament.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessArena {
    /// The tournament id.
    pub id: String,
    /// The full display name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_name: Option<String>,
    /// The creator's username.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    /// The tournament duration in minutes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minutes: Option<u32>,
    /// The clock.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clock: Option<LichessArenaClock>,
    /// Whether the tournament is rated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rated: Option<bool>,
    /// The number of players.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nb_players: Option<u32>,
    /// Start time (Unix milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub starts_at: Option<i64>,
    /// Finish time (Unix milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finishes_at: Option<i64>,
    /// The status code (10 = created, 20 = started, 30 = finished).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<i32>,
    /// Perf display info.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub perf: Option<LichessArenaPerf>,
    /// Seconds until the tournament starts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seconds_to_start: Option<i64>,
    /// The winner, once finished.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub winner: Option<LichessLightUser>,
}

/// Tournaments grouped by lifecycle stage. `GET /api/tournament`
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessArenaList {
    /// Tournaments not yet started.
    #[serde(default)]
    pub created: Vec<LichessArena>,
    /// Tournaments in progress.
    #[serde(default)]
    pub started: Vec<LichessArena>,
    /// Recently finished tournaments.
    #[serde(default)]
    pub finished: Vec<LichessArena>,
}

/// A player row in an arena standings page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessArenaPlayer {
    /// The player's username.
    pub name: String,
    /// The player's title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<LichessTitle>,
    /// The player's current rank.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rank: Option<u32>,
    /// The player's rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rating: Option<u32>,
    /// The player's score.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<u32>,
}

/// A page of arena standings.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessArenaStanding {
    /// The page number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// The players on this page.
    #[serde(default)]
    pub players: Vec<LichessArenaPlayer>,
}

/// Full details of an arena tournament. `GET /api/tournament/{id}`
///
/// Models the commonly-used fields; deeper aggregates (duels, stats, podium)
/// are not decoded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessArenaFull {
    /// The tournament id.
    pub id: String,
    /// The full display name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_name: Option<String>,
    /// Whether the tournament is rated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rated: Option<bool>,
    /// The clock.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clock: Option<LichessArenaClock>,
    /// The duration in minutes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minutes: Option<u32>,
    /// The creator's username.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    /// Seconds until the tournament starts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seconds_to_start: Option<i64>,
    /// Seconds until the tournament finishes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seconds_to_finish: Option<i64>,
    /// Whether the tournament has finished.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_finished: Option<bool>,
    /// Whether pairings are closed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pairings_closed: Option<bool>,
    /// Start time (Unix milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub starts_at: Option<i64>,
    /// The number of players.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nb_players: Option<u32>,
    /// Perf display info.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub perf: Option<LichessArenaPerf>,
    /// The current standings page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub standing: Option<LichessArenaStanding>,
    /// The authenticated user's username, if entered.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub my_username: Option<String>,
}

/// One entry in an arena results stream.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessArenaResult {
    /// The player's final rank.
    pub rank: u32,
    /// The player's score.
    pub score: u32,
    /// The player's rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rating: Option<u32>,
    /// The player's username.
    pub username: String,
    /// The player's title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<LichessTitle>,
    /// The player's tournament performance rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub performance: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_arena_list() {
        let json = r#"{"created":[],"started":[{"id":"abc","fullName":"Hourly",
            "clock":{"limit":300,"increment":0},"nbPlayers":50,"status":20}],
            "finished":[]}"#;
        let list: LichessArenaList = serde_json::from_str(json).unwrap();
        assert_eq!(list.started.len(), 1);
        assert_eq!(list.started[0].nb_players, Some(50));
    }

    #[test]
    fn parses_full_arena_ignoring_unknown_aggregates() {
        let json = r#"{"id":"abc","fullName":"Hourly","nbPlayers":50,
            "standing":{"page":1,"players":[{"name":"A","rank":1,"score":10}]},
            "duels":[{"whatever":true}],"stats":{"games":100}}"#;
        let full: LichessArenaFull = serde_json::from_str(json).unwrap();
        assert_eq!(full.standing.unwrap().players[0].name, "A");
    }
}

/// A team's standing in a team-battle arena.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessTeamBattleTeam {
    /// The team's rank.
    pub rank: u32,
    /// The team id.
    pub id: String,
    /// The team's score.
    pub score: u32,
}

/// The team standings of a team-battle arena.
/// `GET /api/tournament/{id}/teams`
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessTeamBattleStandings {
    /// The tournament id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The teams, best first.
    #[serde(default)]
    pub teams: Vec<LichessTeamBattleTeam>,
}
