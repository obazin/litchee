//! DTOs for the Account concern.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::model::LichessLightUser;

/// A user's game/UI preferences.
///
/// A few common fields are typed; the remaining preference keys (which are a
/// large, evolving set of small scalars) are preserved losslessly in
/// [`other`](Self::other).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessUserPreferences {
    /// The board theme.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    /// The piece set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub piece_set: Option<String>,
    /// The sound set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sound_set: Option<String>,
    /// All other preference keys, preserved verbatim.
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

/// The authenticated user's preferences and language.
/// `GET /api/account/preferences`
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessPreferences {
    /// The preference values.
    #[serde(default)]
    pub prefs: LichessUserPreferences,
    /// The user's language tag (e.g. `"en-GB"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

/// A single entry in the user's timeline.
///
/// The discriminant is in [`entry_type`](Self::entry_type); entry-specific
/// fields are preserved in [`data`](Self::data).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessTimelineEntry {
    /// The entry type (e.g. `"follow"`, `"game-end"`).
    #[serde(rename = "type")]
    pub entry_type: String,
    /// When the entry occurred (Unix milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<i64>,
    /// The entry-specific payload.
    #[serde(flatten)]
    pub data: HashMap<String, Value>,
}

/// The authenticated user's timeline. `GET /api/timeline`
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessTimeline {
    /// The timeline entries.
    #[serde(default)]
    pub entries: Vec<LichessTimelineEntry>,
    /// Light user info for the users referenced by the entries.
    #[serde(default)]
    pub users: HashMap<String, LichessLightUser>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_preferences_with_typed_and_flattened_fields() {
        let json = r#"{"prefs":{"theme":"blue","pieceSet":"cburnett","zen":1,
            "confirmResign":1},"language":"en-GB"}"#;
        let prefs: LichessPreferences = serde_json::from_str(json).unwrap();
        assert_eq!(prefs.prefs.theme.as_deref(), Some("blue"));
        assert_eq!(prefs.language.as_deref(), Some("en-GB"));
        assert_eq!(prefs.prefs.other.get("zen"), Some(&Value::from(1)));
    }

    #[test]
    fn parses_timeline_entry() {
        let json = r#"{"entries":[{"type":"follow","date":1,"u1":"a","u2":"b"}],
            "users":{"a":{"id":"a","name":"A"}}}"#;
        let timeline: LichessTimeline = serde_json::from_str(json).unwrap();
        assert_eq!(timeline.entries[0].entry_type, "follow");
        assert_eq!(timeline.entries[0].data.get("u1"), Some(&Value::from("a")));
        assert_eq!(timeline.users["a"].name, "A");
    }
}
