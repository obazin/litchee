//! DTOs for the Opening Explorer concern.

use serde::{Deserialize, Serialize};

use crate::model::LichessColor;

/// The opening identified for a position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessExplorerOpening {
    /// The ECO code.
    pub eco: String,
    /// The opening name.
    pub name: String,
}

/// A player in an opening-explorer game reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessExplorerGamePlayer {
    /// The player's name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The player's rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rating: Option<u32>,
}

/// A reference to a game in the opening explorer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessExplorerGame {
    /// The game id.
    pub id: String,
    /// The move leading to this game (in top/recent game lists).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uci: Option<String>,
    /// The winner, if any.
    #[serde(default)]
    pub winner: Option<LichessColor>,
    /// The white player.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub white: Option<LichessExplorerGamePlayer>,
    /// The black player.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub black: Option<LichessExplorerGamePlayer>,
    /// The year the game was played.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub year: Option<u32>,
    /// The month the game was played.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub month: Option<String>,
}

/// A candidate move with its aggregate statistics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessExplorerMove {
    /// The move in UCI notation.
    pub uci: String,
    /// The move in SAN notation.
    pub san: String,
    /// Number of games White won.
    pub white: u64,
    /// Number of drawn games.
    pub draws: u64,
    /// Number of games Black won.
    pub black: u64,
    /// The average rating of games with this move.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub average_rating: Option<u32>,
    /// A sample game with this move.
    #[serde(default)]
    pub game: Option<LichessExplorerGame>,
}

/// The opening-explorer result for a position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessExplorerResult {
    /// The identified opening, if any.
    #[serde(default)]
    pub opening: Option<LichessExplorerOpening>,
    /// Number of games White won from this position.
    pub white: u64,
    /// Number of drawn games.
    pub draws: u64,
    /// Number of games Black won.
    pub black: u64,
    /// Candidate moves, most popular first.
    #[serde(default)]
    pub moves: Vec<LichessExplorerMove>,
    /// Notable games from this position.
    #[serde(default)]
    pub top_games: Vec<LichessExplorerGame>,
    /// Recent games from this position (Lichess/player databases).
    #[serde(default)]
    pub recent_games: Vec<LichessExplorerGame>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_explorer_result() {
        let json = r#"{"opening":{"eco":"B01","name":"Scandinavian"},
            "white":100,"draws":40,"black":60,
            "moves":[{"uci":"e2e4","san":"e4","white":50,"draws":20,"black":30,
                      "averageRating":2400}],
            "topGames":[{"id":"g","uci":"e2e4","winner":"white",
                         "white":{"name":"A","rating":2700},"year":2020}]}"#;
        let result: LichessExplorerResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.white, 100);
        assert_eq!(result.moves[0].average_rating, Some(2400));
        assert_eq!(result.top_games[0].winner, Some(LichessColor::White));
    }
}
