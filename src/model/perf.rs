//! Per-variant performance (rating) statistics.

use serde::{Deserialize, Serialize};

/// A player's rating statistics in a single perf (speed or variant).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LichessPerf {
    /// Number of rated games played in this perf.
    pub games: u32,
    /// Current rating.
    pub rating: u32,
    /// Rating deviation.
    pub rd: u32,
    /// Recent rating progression (may be negative).
    pub prog: i32,
    /// Present and `true` only when the rating is provisional.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prov: Option<bool>,
    /// Global ranking; only present for recently active players.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rank: Option<u32>,
}

/// Statistics for the puzzle race-style modes (Storm, Racer, Streak).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LichessPuzzleModePerf {
    /// Number of runs played.
    pub runs: u32,
    /// Best score achieved.
    pub score: u32,
}

/// A player's performance across every perf. All fields are optional because a
/// perf only appears once the player has activity in it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessPerfs {
    /// Chess960.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chess960: Option<LichessPerf>,
    /// Atomic.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub atomic: Option<LichessPerf>,
    /// Racing Kings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub racing_kings: Option<LichessPerf>,
    /// Ultra-bullet.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ultra_bullet: Option<LichessPerf>,
    /// Blitz.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blitz: Option<LichessPerf>,
    /// King of the Hill.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub king_of_the_hill: Option<LichessPerf>,
    /// Three-check.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub three_check: Option<LichessPerf>,
    /// Antichess.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub antichess: Option<LichessPerf>,
    /// Crazyhouse.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crazyhouse: Option<LichessPerf>,
    /// Bullet.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bullet: Option<LichessPerf>,
    /// Correspondence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correspondence: Option<LichessPerf>,
    /// Horde.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub horde: Option<LichessPerf>,
    /// Puzzles.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub puzzle: Option<LichessPerf>,
    /// Classical.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub classical: Option<LichessPerf>,
    /// Rapid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rapid: Option<LichessPerf>,
    /// Puzzle Storm.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storm: Option<LichessPuzzleModePerf>,
    /// Puzzle Racer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub racer: Option<LichessPuzzleModePerf>,
    /// Puzzle Streak.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub streak: Option<LichessPuzzleModePerf>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_perf_with_optional_fields() {
        let json = r#"{"games":109,"rating":1814,"rd":55,"prog":-19,"prov":true}"#;
        let perf: LichessPerf = serde_json::from_str(json).unwrap();
        assert_eq!(perf.rating, 1814);
        assert_eq!(perf.prog, -19);
        assert_eq!(perf.prov, Some(true));
        assert_eq!(perf.rank, None);
    }

    #[test]
    fn maps_camel_case_perf_keys() {
        let json = r#"{"ultraBullet":{"games":1,"rating":1500,"rd":100,"prog":0},
                       "racingKings":{"games":2,"rating":1490,"rd":90,"prog":3}}"#;
        let perfs: LichessPerfs = serde_json::from_str(json).unwrap();
        assert!(perfs.ultra_bullet.is_some());
        assert_eq!(perfs.racing_kings.unwrap().rating, 1490);
        assert!(perfs.bullet.is_none());
    }
}
