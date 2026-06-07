//! DTOs for the External Engine concern.

use serde::{Deserialize, Serialize};

/// A registered external engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessExternalEngine {
    /// The unique registration id.
    pub id: String,
    /// The engine's display name.
    pub name: String,
    /// The secret used to request analysis from this engine.
    pub client_secret: String,
    /// The user the engine is registered for.
    pub user_id: String,
    /// Maximum number of threads.
    pub max_threads: u32,
    /// Maximum hash table size, in MiB.
    pub max_hash: u32,
    /// Supported variants (UCI variant names).
    #[serde(default)]
    pub variants: Vec<String>,
    /// Arbitrary provider bookkeeping data.
    #[serde(default)]
    pub provider_data: Option<String>,
}

/// The registration body for creating or updating an external engine.
///
/// This is a request input, so it is exhaustive and constructible by callers
/// (use `..Default::default()` for the optional fields).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LichessExternalEngineRegistration {
    /// The engine's display name (3–200 characters).
    pub name: String,
    /// Maximum number of threads.
    pub max_threads: u32,
    /// Maximum hash table size, in MiB.
    pub max_hash: u32,
    /// A secret shared with the engine provider.
    pub provider_secret: String,
    /// Default search depth.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_depth: Option<u32>,
    /// Supported variants (UCI variant names).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub variants: Vec<String>,
    /// Arbitrary provider bookkeeping data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_data: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_external_engine() {
        let json = r#"{"id":"eng","name":"My Engine","clientSecret":"s","userId":"u",
            "maxThreads":8,"maxHash":256,"variants":["chess"]}"#;
        let engine: LichessExternalEngine = serde_json::from_str(json).unwrap();
        assert_eq!(engine.id, "eng");
        assert_eq!(engine.max_threads, 8);
        assert_eq!(engine.variants, vec!["chess"]);
    }

    #[test]
    fn registration_skips_empty_optional_fields() {
        let registration = LichessExternalEngineRegistration {
            name: "E".to_owned(),
            max_threads: 4,
            max_hash: 128,
            provider_secret: "secret".to_owned(),
            ..Default::default()
        };
        let json = serde_json::to_string(&registration).unwrap();
        assert!(!json.contains("variants"));
        assert!(!json.contains("defaultDepth"));
    }
}

/// A unit of analysis work acquired by an engine provider.
/// `POST /api/external-engine/work`
#[derive(Debug, Clone, Default, PartialEq, Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct LichessEngineWork {
    /// The work id, used to submit results.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// All other fields (the work specification), preserved verbatim.
    #[serde(flatten)]
    pub other: std::collections::HashMap<String, serde_json::Value>,
}
