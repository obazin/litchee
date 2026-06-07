//! DTOs for the Broadcasts concern.
//!
//! Tournament and round listings are typed; the deeply-nested round/games/
//! player payloads keep their richer fields in lossless `Value` maps.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A broadcast tournament's metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessBroadcastTour {
    /// The tournament id.
    pub id: String,
    /// The tournament name.
    pub name: String,
    /// The URL slug.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    /// The description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The canonical URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// The promotion tier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tier: Option<i32>,
    /// Creation time (Unix milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    /// The cover image URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
}

/// A round within a broadcast tournament.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessBroadcastRoundInfo {
    /// The round id.
    pub id: String,
    /// The round name.
    pub name: String,
    /// The URL slug.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    /// The canonical URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Creation time (Unix milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    /// Whether the round is rated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rated: Option<bool>,
    /// Whether the round is currently ongoing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ongoing: Option<bool>,
    /// Start time (Unix milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub starts_at: Option<i64>,
    /// Finish time (Unix milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<i64>,
    /// Whether the round has finished.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished: Option<bool>,
}

/// A broadcast tournament together with its rounds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessBroadcast {
    /// The tournament.
    pub tour: LichessBroadcastTour,
    /// The group this tournament belongs to, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// The rounds.
    #[serde(default)]
    pub rounds: Vec<LichessBroadcastRoundInfo>,
    /// The id of the round shown by default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_round_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_broadcast_with_rounds() {
        let json = r#"{"tour":{"id":"abc","name":"World Champ","slug":"wc"},
            "rounds":[{"id":"r1","name":"Round 1","slug":"round-1","url":"u",
                       "createdAt":1,"rated":true,"finished":false}]}"#;
        let broadcast: LichessBroadcast = serde_json::from_str(json).unwrap();
        assert_eq!(broadcast.tour.name, "World Champ");
        assert_eq!(broadcast.rounds[0].id, "r1");
        assert_eq!(broadcast.rounds[0].finished, Some(false));
    }
}

/// The top broadcasts. `GET /api/broadcast/top`
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessBroadcastTop {
    /// Currently-active broadcasts.
    #[serde(default)]
    pub active: Vec<LichessBroadcast>,
    /// Upcoming broadcasts.
    #[serde(default)]
    pub upcoming: Vec<LichessBroadcast>,
    /// A page of past broadcasts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub past: Option<Value>,
}

/// A paginated page of broadcast search results.
/// `GET /api/broadcast/search`
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessBroadcastSearchPage {
    /// The current page number.
    pub current_page: u32,
    /// Maximum results per page.
    pub max_per_page: u32,
    /// The broadcasts on this page.
    #[serde(default)]
    pub current_page_results: Vec<LichessBroadcast>,
    /// The previous page number, if any.
    #[serde(default)]
    pub previous_page: Option<u32>,
    /// The next page number, if any.
    #[serde(default)]
    pub next_page: Option<u32>,
}

/// A round with its games. `GET /api/broadcast/{tourSlug}/{roundSlug}/{roundId}`
///
/// The deeply-nested round/study/games payloads are preserved verbatim.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessBroadcastRoundView {
    /// The parent tournament.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tour: Option<LichessBroadcastTour>,
    /// The round info.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub round: Option<Value>,
    /// The games in the round.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub games: Option<Value>,
    /// The backing study info.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub study: Option<Value>,
}

/// A player entry in a broadcast. `GET /broadcast/{id}/players`
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessBroadcastPlayerEntry {
    /// The player's name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// All other fields, preserved verbatim.
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

/// One of the authenticated user's broadcast rounds.
/// `GET /api/broadcast/my-rounds`
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessBroadcastMyRound {
    /// All fields, preserved verbatim (round + tour + study).
    #[serde(flatten)]
    pub data: HashMap<String, Value>,
}

/// The result of pushing PGN to a round.
/// `POST /api/broadcast/round/{roundId}/push`
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessBroadcastPushResult {
    /// All fields, preserved verbatim.
    #[serde(flatten)]
    pub data: HashMap<String, Value>,
}
