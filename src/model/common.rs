//! Cross-cutting primitive types used across many concerns.

use serde::{Deserialize, Serialize};

/// A side of the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LichessColor {
    /// The white pieces.
    White,
    /// The black pieces.
    Black,
}

/// A time-control speed category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum LichessSpeed {
    /// Ultra-bullet (extremely fast).
    UltraBullet,
    /// Bullet.
    Bullet,
    /// Blitz.
    Blitz,
    /// Rapid.
    Rapid,
    /// Classical.
    Classical,
    /// Correspondence (days per move).
    Correspondence,
}

/// A chess variant key as used throughout the API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum LichessVariantKey {
    /// Standard chess.
    Standard,
    /// Chess960 (Fischer random).
    Chess960,
    /// Crazyhouse.
    Crazyhouse,
    /// Antichess.
    Antichess,
    /// Atomic.
    Atomic,
    /// Horde.
    Horde,
    /// King of the Hill.
    KingOfTheHill,
    /// Racing Kings.
    RacingKings,
    /// Three-check.
    ThreeCheck,
    /// A game started from a custom position.
    FromPosition,
}

/// The trivial `{ "ok": true }` acknowledgement some endpoints return.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LichessOk {
    /// Always `true` on success.
    pub ok: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_uses_lowercase() {
        assert_eq!(
            serde_json::to_string(&LichessColor::Black).unwrap(),
            "\"black\""
        );
    }

    #[test]
    fn speed_uses_camel_case() {
        assert_eq!(
            serde_json::from_str::<LichessSpeed>("\"ultraBullet\"").unwrap(),
            LichessSpeed::UltraBullet
        );
    }

    #[test]
    fn variant_key_uses_camel_case() {
        assert_eq!(
            serde_json::to_string(&LichessVariantKey::KingOfTheHill).unwrap(),
            "\"kingOfTheHill\""
        );
        assert_eq!(
            serde_json::from_str::<LichessVariantKey>("\"chess960\"").unwrap(),
            LichessVariantKey::Chess960
        );
    }
}
