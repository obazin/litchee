//! DTOs for the Studies concern.

use serde::{Deserialize, Serialize};

/// Metadata about a study.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessStudyMetadata {
    /// The study id.
    pub id: String,
    /// The study name.
    pub name: String,
    /// Creation time (Unix milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    /// Last-update time (Unix milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<i64>,
}

/// A player named in an imported chapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessStudyChapterPlayer {
    /// The player's name.
    #[serde(default)]
    pub name: Option<String>,
    /// The player's rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rating: Option<u32>,
}

/// A chapter created by importing PGN.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessStudyChapter {
    /// The chapter id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The chapter name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The two players, if the chapter is a game.
    #[serde(default)]
    pub players: Vec<LichessStudyChapterPlayer>,
    /// The chapter status / result.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// The result of importing PGN into a study.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessStudyImportResult {
    /// The chapters that were created.
    #[serde(default)]
    pub chapters: Vec<LichessStudyChapter>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_study_metadata() {
        let json = r#"{"id":"WTvnkWAL","name":"Guess the move",
            "createdAt":1463756350225,"updatedAt":1469965025205}"#;
        let meta: LichessStudyMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(meta.id, "WTvnkWAL");
        assert_eq!(meta.updated_at, Some(1_469_965_025_205));
    }

    #[test]
    fn parses_import_result() {
        let json = r#"{"chapters":[{"id":"iBjmYBya","name":"test 2",
            "players":[{"name":"Carlsen, Magnus","rating":2837},
                       {"name":null,"rating":2580}],"status":"1-0"}]}"#;
        let result: LichessStudyImportResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.chapters[0].id.as_deref(), Some("iBjmYBya"));
        assert_eq!(result.chapters[0].players[1].name, None);
    }
}
