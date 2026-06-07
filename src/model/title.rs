//! Player titles.

use serde::{Deserialize, Serialize};

/// A player's official title, or `BOT` for bot accounts.
///
/// Only present for titled players and bots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
#[non_exhaustive]
pub enum LichessTitle {
    /// Grandmaster.
    Gm,
    /// Woman Grandmaster.
    Wgm,
    /// International Master.
    Im,
    /// Woman International Master.
    Wim,
    /// FIDE Master.
    Fm,
    /// Woman FIDE Master.
    Wfm,
    /// National Master.
    Nm,
    /// Candidate Master.
    Cm,
    /// Woman Candidate Master.
    Wcm,
    /// Woman National Master.
    Wnm,
    /// Lichess Master.
    Lm,
    /// A bot account.
    Bot,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_to_uppercase_codes() {
        assert_eq!(serde_json::to_string(&LichessTitle::Gm).unwrap(), "\"GM\"");
        assert_eq!(
            serde_json::to_string(&LichessTitle::Wgm).unwrap(),
            "\"WGM\""
        );
        assert_eq!(
            serde_json::to_string(&LichessTitle::Bot).unwrap(),
            "\"BOT\""
        );
    }

    #[test]
    fn deserializes_from_uppercase_codes() {
        assert_eq!(
            serde_json::from_str::<LichessTitle>("\"WIM\"").unwrap(),
            LichessTitle::Wim
        );
        assert_eq!(
            serde_json::from_str::<LichessTitle>("\"LM\"").unwrap(),
            LichessTitle::Lm
        );
    }
}
