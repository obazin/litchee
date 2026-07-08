//! Shared annotated-PGN export options.
//!
//! The study and broadcast PGN export endpoints accept the same block of
//! optional flags controlling which annotations the PGN carries.
//! [`PgnExportOptions`] models that block once so each concern can pass it as a
//! query without redeclaring the fields. Broadcast exports accept only
//! `clocks`/`comments`; Lichess ignores the flags an endpoint does not support.

use serde::Serialize;

/// Optional annotation flags shared by the study/broadcast PGN exports.
///
/// Every field is unset by default, so a [`PgnExportOptions::default`] adds
/// nothing to the query and each endpoint keeps its own server defaults.
#[derive(Debug, Clone, Default, Serialize)]
pub struct PgnExportOptions {
    /// Include clock comments, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    clocks: Option<bool>,
    /// Include analysis and annotator comments (e.g. `[%eval ...]`).
    #[serde(skip_serializing_if = "Option::is_none")]
    comments: Option<bool>,
    /// Include analysis variations.
    #[serde(skip_serializing_if = "Option::is_none")]
    variations: Option<bool>,
    /// Include the board orientation.
    #[serde(skip_serializing_if = "Option::is_none")]
    orientation: Option<bool>,
}

impl PgnExportOptions {
    /// Include clock comments, when available.
    #[must_use]
    pub fn clocks(mut self, include: bool) -> Self {
        self.clocks = Some(include);
        self
    }

    /// Include analysis and annotator comments (e.g. `[%eval ...]`).
    #[must_use]
    pub fn comments(mut self, include: bool) -> Self {
        self.comments = Some(include);
        self
    }

    /// Include analysis variations.
    #[must_use]
    pub fn variations(mut self, include: bool) -> Self {
        self.variations = Some(include);
        self
    }

    /// Include the board orientation.
    #[must_use]
    pub fn orientation(mut self, include: bool) -> Self {
        self.orientation = Some(include);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_serializes_to_nothing() {
        assert_eq!(
            serde_urlencoded::to_string(PgnExportOptions::default()).unwrap(),
            ""
        );
    }

    #[test]
    fn set_flags_serialize() {
        let query = serde_urlencoded::to_string(
            PgnExportOptions::default().clocks(true).orientation(false),
        )
        .unwrap();
        assert_eq!(query, "clocks=true&orientation=false");
    }
}
