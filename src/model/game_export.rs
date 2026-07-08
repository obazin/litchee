//! Shared game-export formatting options.
//!
//! The Lichess API exposes the same block of optional query flags on every
//! game-returning endpoint — game export, a user's games, bookmarks, tournament
//! games, TV games, etc. [`GameExportOptions`] models that block once so each
//! concern can layer it onto its own request without redeclaring the fields.

use serde::Serialize;

/// Optional formatting flags shared by the game-export endpoints.
///
/// Every field is unset by default, so a [`GameExportOptions::default`] adds
/// nothing to the query and each endpoint keeps its own server-side defaults.
/// Set only the flags you want to override. Some endpoints accept a subset
/// (e.g. TV ignores `evals`/`accuracy`/`division`/`literate`); Lichess simply
/// ignores flags an endpoint does not support.
///
/// Layer it onto a request builder's query — reqwest appends repeated `query`
/// calls, so `request.query(&filters).query(&options)` sends both.
#[derive(Debug, Clone, Default, Serialize)]
pub struct GameExportOptions {
    /// Include the PGN moves (`true`) or only headers (`false`).
    #[serde(skip_serializing_if = "Option::is_none")]
    moves: Option<bool>,
    /// Include the full PGN within the JSON representation.
    #[serde(rename = "pgnInJson", skip_serializing_if = "Option::is_none")]
    pgn_in_json: Option<bool>,
    /// Include the PGN tags.
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<bool>,
    /// Include clock comments in the PGN move list, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    clocks: Option<bool>,
    /// Include analysis evaluation comments, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    evals: Option<bool>,
    /// Include accuracy percentages, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    accuracy: Option<bool>,
    /// Include the opening name.
    #[serde(skip_serializing_if = "Option::is_none")]
    opening: Option<bool>,
    /// Include the game division (opening/middlegame/endgame plies).
    #[serde(skip_serializing_if = "Option::is_none")]
    division: Option<bool>,
    /// Insert textual annotations for a more human-readable PGN.
    #[serde(skip_serializing_if = "Option::is_none")]
    literate: Option<bool>,
}

impl GameExportOptions {
    /// Include the PGN moves (`true`) or only headers (`false`).
    #[must_use]
    pub fn moves(mut self, include: bool) -> Self {
        self.moves = Some(include);
        self
    }

    /// Include the full PGN within the JSON representation.
    #[must_use]
    pub fn pgn_in_json(mut self, include: bool) -> Self {
        self.pgn_in_json = Some(include);
        self
    }

    /// Include the PGN tags.
    #[must_use]
    pub fn tags(mut self, include: bool) -> Self {
        self.tags = Some(include);
        self
    }

    /// Include clock comments in the PGN move list, when available.
    #[must_use]
    pub fn clocks(mut self, include: bool) -> Self {
        self.clocks = Some(include);
        self
    }

    /// Include analysis evaluation comments, when available.
    #[must_use]
    pub fn evals(mut self, include: bool) -> Self {
        self.evals = Some(include);
        self
    }

    /// Include accuracy percentages, when available.
    #[must_use]
    pub fn accuracy(mut self, include: bool) -> Self {
        self.accuracy = Some(include);
        self
    }

    /// Include the opening name.
    #[must_use]
    pub fn opening(mut self, include: bool) -> Self {
        self.opening = Some(include);
        self
    }

    /// Include the game division (opening/middlegame/endgame plies).
    #[must_use]
    pub fn division(mut self, include: bool) -> Self {
        self.division = Some(include);
        self
    }

    /// Insert textual annotations for a more human-readable PGN.
    #[must_use]
    pub fn literate(mut self, include: bool) -> Self {
        self.literate = Some(include);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_serializes_to_nothing() {
        let query = serde_urlencoded::to_string(GameExportOptions::default()).unwrap();
        assert_eq!(query, "");
    }

    #[test]
    fn set_flags_use_their_wire_names() {
        let options = GameExportOptions::default()
            .moves(true)
            .pgn_in_json(true)
            .clocks(false)
            .division(true);
        let query = serde_urlencoded::to_string(&options).unwrap();
        assert_eq!(
            query,
            "moves=true&pgnInJson=true&clocks=false&division=true"
        );
    }

    #[test]
    fn unset_flags_are_omitted() {
        let query = serde_urlencoded::to_string(GameExportOptions::default().evals(true)).unwrap();
        assert_eq!(query, "evals=true");
    }
}
