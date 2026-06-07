//! DTOs for the Swiss Tournaments concern.

use serde::{Deserialize, Serialize};

use crate::model::LichessTitle;

/// A swiss clock (`limit` + `increment`, in seconds).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessSwissClock {
    /// Initial time in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Increment per move in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub increment: Option<u32>,
}

/// When the next round of a swiss starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessSwissNextRound {
    /// Absolute start time (Unix milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub at: Option<i64>,
    /// Seconds until the next round starts.
    #[serde(rename = "in", default, skip_serializing_if = "Option::is_none")]
    pub in_seconds: Option<i64>,
}

/// A swiss tournament.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessSwiss {
    /// The tournament id.
    pub id: String,
    /// The creator's username.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    /// Start time (Unix milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub starts_at: Option<i64>,
    /// The tournament name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The clock.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clock: Option<LichessSwissClock>,
    /// The variant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    /// The current round number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub round: Option<u32>,
    /// The total number of rounds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nb_rounds: Option<u32>,
    /// The number of players.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nb_players: Option<u32>,
    /// The number of ongoing games.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nb_ongoing: Option<u32>,
    /// The status (`created`, `started`, or `finished`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Whether the tournament is rated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rated: Option<bool>,
    /// When the next round starts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_round: Option<LichessSwissNextRound>,
}

/// One entry in a swiss results stream.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessSwissResult {
    /// The player's final rank.
    pub rank: u32,
    /// The player's points.
    pub points: f64,
    /// The player's tie-break score.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tie_break: Option<f64>,
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
    fn parses_swiss_with_next_round() {
        let json = r#"{"id":"abc","name":"Weekly","clock":{"limit":300,"increment":0},
            "variant":"standard","round":2,"nbRounds":7,"nbPlayers":40,
            "status":"started","rated":true,"nextRound":{"at":1700000000000,"in":120}}"#;
        let swiss: LichessSwiss = serde_json::from_str(json).unwrap();
        assert_eq!(swiss.nb_rounds, Some(7));
        assert_eq!(swiss.next_round.unwrap().in_seconds, Some(120));
    }

    #[test]
    fn parses_swiss_result_with_fractional_points() {
        let json = r#"{"rank":1,"points":5.5,"tieBreak":18.0,"username":"A","rating":2400}"#;
        let result: LichessSwissResult = serde_json::from_str(json).unwrap();
        assert!((result.points - 5.5).abs() < f64::EPSILON);
    }
}
