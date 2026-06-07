//! DTOs for the Tablebase concern.
//!
//! Field names already match the API's `snake_case` JSON, so no renaming is
//! needed here.

use serde::{Deserialize, Serialize};

/// The theoretical result category of a position or move.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum LichessTablebaseCategory {
    /// A win.
    Win,
    /// Unknown result.
    Unknown,
    /// A win, but the exact result is uncertain due to DTZ rounding.
    SyzygyWin,
    /// Possibly a win, with respect to the 50-move rule.
    MaybeWin,
    /// A win prevented from being decisive by the 50-move rule.
    CursedWin,
    /// A draw.
    Draw,
    /// A loss prevented from being decisive by the 50-move rule.
    BlessedLoss,
    /// Possibly a loss, with respect to the 50-move rule.
    MaybeLoss,
    /// A loss, but the exact result is uncertain due to DTZ rounding.
    SyzygyLoss,
    /// A loss.
    Loss,
}

/// A legal move with its tablebase evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessTablebaseMove {
    /// The move in UCI notation.
    pub uci: String,
    /// The move in SAN notation.
    pub san: String,
    /// The resulting position's category.
    pub category: LichessTablebaseCategory,
    /// Whether the move zeroes the 50-move counter (capture or pawn move).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zeroing: Option<bool>,
    /// Distance to zeroing (DTZ50'' with rounding), in plies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dtz: Option<i32>,
    /// Precise DTZ, only when guaranteed not rounded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub precise_dtz: Option<i32>,
    /// Depth to conversion, in plies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dtc: Option<i32>,
    /// Depth to mate, in plies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dtm: Option<i32>,
    /// Depth to win, in plies (variants only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dtw: Option<i32>,
    /// Whether this is a checkmate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkmate: Option<bool>,
    /// Whether this is a stalemate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stalemate: Option<bool>,
    /// Whether this is a variant win.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant_win: Option<bool>,
    /// Whether this is a variant loss.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant_loss: Option<bool>,
    /// Whether there is insufficient material.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub insufficient_material: Option<bool>,
}

/// Tablebase information about a position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessTablebasePosition {
    /// The position's result category.
    pub category: LichessTablebaseCategory,
    /// Legal moves, best first.
    pub moves: Vec<LichessTablebaseMove>,
    /// Distance to zeroing (DTZ50'' with rounding), in plies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dtz: Option<i32>,
    /// Precise DTZ, only when guaranteed not rounded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub precise_dtz: Option<i32>,
    /// Depth to conversion, in plies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dtc: Option<i32>,
    /// Depth to mate, in plies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dtm: Option<i32>,
    /// Depth to win, in plies (variants only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dtw: Option<i32>,
    /// Whether the position is checkmate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkmate: Option<bool>,
    /// Whether the position is stalemate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stalemate: Option<bool>,
    /// Whether the position is a variant win.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant_win: Option<bool>,
    /// Whether the position is a variant loss.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant_loss: Option<bool>,
    /// Whether there is insufficient material.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub insufficient_material: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_position_with_kebab_case_category() {
        let json = r#"{"category":"cursed-win","dtz":1,"dtm":17,"checkmate":false,
            "moves":[{"uci":"h7h8q","san":"h8=Q+","category":"loss","zeroing":true}]}"#;
        let position: LichessTablebasePosition = serde_json::from_str(json).unwrap();
        assert_eq!(position.category, LichessTablebaseCategory::CursedWin);
        assert_eq!(position.dtz, Some(1));
        assert_eq!(position.moves[0].category, LichessTablebaseCategory::Loss);
        assert_eq!(position.moves[0].zeroing, Some(true));
    }
}
