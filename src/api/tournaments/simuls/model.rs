//! DTOs for the Simuls concern.

use serde::{Deserialize, Serialize};

use crate::model::{LichessLightUser, LichessVariantKey};

/// The host of a simul: a light user with simul-specific fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessSimulHost {
    /// The host's light user info.
    #[serde(flatten)]
    pub user: LichessLightUser,
    /// The host's rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rating: Option<u32>,
    /// Whether the host's rating is provisional.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provisional: Option<bool>,
    /// The host's current game id, if playing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game_id: Option<String>,
    /// Whether the host is online.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub online: Option<bool>,
}

/// A variant offered in a simul.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessSimulVariant {
    /// The variant key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<LichessVariantKey>,
    /// The display icon.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// The display name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// A simultaneous exhibition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessSimul {
    /// The simul id.
    pub id: String,
    /// The host.
    pub host: LichessSimulHost,
    /// Short name.
    pub name: String,
    /// Full name.
    pub full_name: String,
    /// Offered variants.
    pub variants: Vec<LichessSimulVariant>,
    /// Whether the simul is created but not yet started.
    pub is_created: bool,
    /// Whether the simul has finished.
    pub is_finished: bool,
    /// Whether the simul is in progress.
    pub is_running: bool,
    /// The host's description text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Estimated start time (Unix milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimated_start_at: Option<i64>,
    /// Actual start time (Unix milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_at: Option<i64>,
    /// Finish time (Unix milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<i64>,
    /// Number of applicants.
    pub nb_applicants: u32,
    /// Number of pairings.
    pub nb_pairings: u32,
}

/// The simuls grouped by lifecycle stage.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessSimuls {
    /// Your created-but-unstarted simuls (only when authenticated).
    #[serde(default)]
    pub pending: Vec<LichessSimul>,
    /// Recently created simuls.
    #[serde(default)]
    pub created: Vec<LichessSimul>,
    /// Currently running simuls.
    #[serde(default)]
    pub started: Vec<LichessSimul>,
    /// Recently finished simuls.
    #[serde(default)]
    pub finished: Vec<LichessSimul>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simul_with_flattened_host() {
        let json = r#"{
            "id":"abc","name":"Sunday","fullName":"Sunday Simul",
            "host":{"id":"bobby","name":"Bobby","rating":2400,"online":true},
            "variants":[{"key":"standard","name":"Standard"}],
            "isCreated":true,"isFinished":false,"isRunning":false,
            "nbApplicants":3,"nbPairings":0
        }"#;
        let simul: LichessSimul = serde_json::from_str(json).unwrap();
        assert_eq!(simul.host.user.id, "bobby");
        assert_eq!(simul.host.rating, Some(2400));
        assert_eq!(simul.variants[0].key, Some(LichessVariantKey::Standard));
        assert!(simul.is_created);
    }

    #[test]
    fn simuls_default_missing_groups_to_empty() {
        let simuls: LichessSimuls = serde_json::from_str(r#"{"started":[]}"#).unwrap();
        assert!(simuls.pending.is_empty());
        assert!(simuls.finished.is_empty());
    }
}
