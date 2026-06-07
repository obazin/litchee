//! DTOs for the Bulk Pairing concern.

use serde::{Deserialize, Serialize};

use crate::model::LichessVariantKey;

/// A clock as used by bulk pairings (`limit` + `increment`, in seconds).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessBulkClock {
    /// Initial time in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Increment per move in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub increment: Option<u32>,
}

/// A single pairing within a bulk pairing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessBulkPairingGame {
    /// The game id.
    pub id: String,
    /// The white player's id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub white: Option<String>,
    /// The black player's id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub black: Option<String>,
}

/// A bulk pairing: a batch of games created together.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessBulkPairing {
    /// The bulk pairing id.
    pub id: String,
    /// The paired games.
    #[serde(default)]
    pub games: Vec<LichessBulkPairingGame>,
    /// The variant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<LichessVariantKey>,
    /// The clock.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clock: Option<LichessBulkClock>,
    /// When the games will be created (Unix milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pair_at: Option<i64>,
    /// When the games were created, if already paired.
    #[serde(default)]
    pub paired_at: Option<i64>,
    /// Whether the games are rated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rated: Option<bool>,
    /// When the clocks will start (Unix milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_clocks_at: Option<i64>,
    /// When the pairing was scheduled (Unix milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scheduled_at: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bulk_pairing_with_null_paired_at() {
        let json = r#"{"id":"RVAcwgg7",
            "games":[{"id":"NKop9IyD","black":"lizen1","white":"thibault"}],
            "variant":"standard","clock":{"increment":0,"limit":300},
            "pairAt":1612289869919,"pairedAt":null,"rated":false,
            "startClocksAt":1612200422971,"scheduledAt":1612203514628}"#;
        let pairing: LichessBulkPairing = serde_json::from_str(json).unwrap();
        assert_eq!(pairing.games.len(), 1);
        assert_eq!(pairing.paired_at, None);
        assert_eq!(pairing.clock.unwrap().limit, Some(300));
    }
}
