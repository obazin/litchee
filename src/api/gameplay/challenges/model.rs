//! DTOs for the Challenges concern.

use serde::{Deserialize, Serialize};

use crate::model::{LichessColor, LichessSpeed, LichessTitle, LichessVariantKey};

/// The lifecycle status of a challenge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum LichessChallengeStatus {
    /// Created and pending.
    Created,
    /// The destination user is offline.
    Offline,
    /// Canceled by the challenger.
    Canceled,
    /// Declined by the destination user.
    Declined,
    /// Accepted; a game has started.
    Accepted,
}

/// The requested color of a challenge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum LichessChallengeColor {
    /// Play as white.
    White,
    /// Play as black.
    Black,
    /// Random color.
    Random,
}

/// A user involved in a challenge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessChallengeUser {
    /// The user id.
    pub id: String,
    /// The display name.
    pub name: String,
    /// The user's rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rating: Option<u32>,
    /// The user's title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<LichessTitle>,
    /// Whether the rating is provisional.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provisional: Option<bool>,
    /// Whether the user is online.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub online: Option<bool>,
    /// The user's network lag in milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lag: Option<u32>,
}

/// The time control of a challenge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
#[non_exhaustive]
pub enum LichessTimeControl {
    /// A real-time clock.
    Clock {
        /// Initial time in seconds.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        limit: Option<u32>,
        /// Increment per move in seconds.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        increment: Option<u32>,
        /// Human-readable form (e.g. `"5+2"`).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        show: Option<String>,
    },
    /// A correspondence time control.
    Correspondence {
        /// Days per turn.
        #[serde(
            rename = "daysPerTurn",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        days_per_turn: Option<u32>,
    },
    /// No time limit.
    Unlimited,
}

/// Display info for a challenge's perf.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessChallengePerf {
    /// The perf icon.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// The perf name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Variant details for a challenge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessChallengeVariant {
    /// The variant key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<LichessVariantKey>,
    /// The full variant name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// A short variant name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub short: Option<String>,
}

/// A challenge between two players.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessChallenge {
    /// The challenge id.
    pub id: String,
    /// The challenge URL.
    pub url: String,
    /// The status.
    pub status: LichessChallengeStatus,
    /// The challenger (absent for open challenges).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub challenger: Option<LichessChallengeUser>,
    /// The destination user (absent for open challenges).
    #[serde(default)]
    pub dest_user: Option<LichessChallengeUser>,
    /// The variant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<LichessChallengeVariant>,
    /// Whether the game would be rated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rated: Option<bool>,
    /// The speed category.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed: Option<LichessSpeed>,
    /// The time control.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_control: Option<LichessTimeControl>,
    /// The requested color.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<LichessChallengeColor>,
    /// The resolved color, if random has been settled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub final_color: Option<LichessColor>,
    /// Perf display info.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub perf: Option<LichessChallengePerf>,
    /// The direction (`in` or `out`) relative to the authenticated user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    /// The initial FEN, for custom positions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_fen: Option<String>,
    /// The id of the game this is a rematch of.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rematch_of: Option<String>,
}

/// The incoming and outgoing challenges for the authenticated user.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessChallenges {
    /// Challenges sent to the user.
    #[serde(rename = "in", default)]
    pub incoming: Vec<LichessChallenge>,
    /// Challenges sent by the user.
    #[serde(rename = "out", default)]
    pub outgoing: Vec<LichessChallenge>,
}

/// An open challenge that anyone can accept.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessOpenChallenge {
    /// The challenge id.
    pub id: String,
    /// The challenge URL.
    pub url: String,
    /// The status.
    pub status: LichessChallengeStatus,
    /// Whether the game would be rated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rated: Option<bool>,
    /// The variant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<LichessChallengeVariant>,
    /// The time control.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_control: Option<LichessTimeControl>,
    /// URL for the player taking white.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url_white: Option<String>,
    /// URL for the player taking black.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url_black: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_challenge_with_clock_time_control() {
        let json = r#"{"id":"H9fIRZUk","url":"https://lichess.org/H9fIRZUk",
            "status":"created","challenger":{"id":"bot1","name":"Bot1","rating":1500},
            "destUser":{"id":"bobby","name":"Bobby"},"rated":true,"speed":"rapid",
            "timeControl":{"type":"clock","limit":600,"increment":0,"show":"10+0"},
            "color":"random","finalColor":"black","direction":"out"}"#;
        let challenge: LichessChallenge = serde_json::from_str(json).unwrap();
        assert_eq!(challenge.id, "H9fIRZUk");
        assert_eq!(challenge.color, Some(LichessChallengeColor::Random));
        assert_eq!(
            challenge.time_control,
            Some(LichessTimeControl::Clock {
                limit: Some(600),
                increment: Some(0),
                show: Some("10+0".to_owned()),
            })
        );
    }

    #[test]
    fn parses_challenge_list_in_out() {
        let json = r#"{"in":[],"out":[{"id":"x","url":"u","status":"created"}]}"#;
        let challenges: LichessChallenges = serde_json::from_str(json).unwrap();
        assert!(challenges.incoming.is_empty());
        assert_eq!(challenges.outgoing.len(), 1);
    }
}
