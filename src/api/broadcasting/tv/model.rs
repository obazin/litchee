//! DTOs for the TV concern.

use serde::{Deserialize, Serialize};

use crate::model::{LichessColor, LichessLightUser};

/// A featured game on a TV channel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessTvGame {
    /// The featured player.
    pub user: LichessLightUser,
    /// The featured player's rating.
    pub rating: i32,
    /// The game id.
    pub game_id: String,
    /// The featured player's color.
    pub color: LichessColor,
}

/// The current best game for each TV channel.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessTvChannels {
    /// The best bot game.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bot: Option<LichessTvGame>,
    /// The best blitz game.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blitz: Option<LichessTvGame>,
    /// The best Racing Kings game.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub racing_kings: Option<LichessTvGame>,
    /// The best ultra-bullet game.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ultra_bullet: Option<LichessTvGame>,
    /// The best bullet game.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bullet: Option<LichessTvGame>,
    /// The best classical game.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub classical: Option<LichessTvGame>,
    /// The best three-check game.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub three_check: Option<LichessTvGame>,
    /// The best antichess game.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub antichess: Option<LichessTvGame>,
    /// The best computer game.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub computer: Option<LichessTvGame>,
    /// The best horde game.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub horde: Option<LichessTvGame>,
    /// The best rapid game.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rapid: Option<LichessTvGame>,
    /// The best atomic game.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub atomic: Option<LichessTvGame>,
    /// The best crazyhouse game.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crazyhouse: Option<LichessTvGame>,
    /// The best Chess960 game.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chess960: Option<LichessTvGame>,
    /// The best King of the Hill game.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub king_of_the_hill: Option<LichessTvGame>,
    /// The overall best game.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub best: Option<LichessTvGame>,
}

/// A player in a featured TV game.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessTvFeedPlayer {
    /// The player's color.
    pub color: LichessColor,
    /// The player's light user info.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<LichessLightUser>,
    /// The player's rating.
    pub rating: i32,
    /// The player's remaining time in seconds.
    pub seconds: i32,
}

/// The summary message sent when a featured game starts or changes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessTvFeatured {
    /// The game id.
    pub id: String,
    /// The board orientation.
    pub orientation: LichessColor,
    /// The two players.
    pub players: Vec<LichessTvFeedPlayer>,
    /// The current position (X-FEN).
    pub fen: String,
}

/// An incremental position update for the featured game.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessTvMove {
    /// The current position (X-FEN).
    pub fen: String,
    /// The last move in UCI notation.
    pub lm: String,
    /// White's clock in seconds.
    pub wc: i32,
    /// Black's clock in seconds.
    pub bc: i32,
}

/// An event from a TV feed stream.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "t", content = "d")]
#[non_exhaustive]
pub enum LichessTvFeedEvent {
    /// A full summary of the featured game (sent first and on changes).
    #[serde(rename = "featured")]
    Featured(LichessTvFeatured),
    /// An incremental move update.
    #[serde(rename = "fen")]
    Fen(LichessTvMove),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_channels() {
        let json = r#"{"bullet":{"user":{"id":"a","name":"A"},"rating":2900,
                       "gameId":"x","color":"white"}}"#;
        let channels: LichessTvChannels = serde_json::from_str(json).unwrap();
        assert_eq!(channels.bullet.unwrap().game_id, "x");
        assert!(channels.blitz.is_none());
    }

    #[test]
    fn parses_featured_feed_event() {
        let json = r#"{"t":"featured","d":{"id":"g","orientation":"white",
            "players":[{"color":"white","rating":1500,"seconds":60}],"fen":"startpos"}}"#;
        let event: LichessTvFeedEvent = serde_json::from_str(json).unwrap();
        match event {
            LichessTvFeedEvent::Featured(f) => assert_eq!(f.id, "g"),
            LichessTvFeedEvent::Fen(_) => panic!("expected featured"),
        }
    }

    #[test]
    fn parses_fen_feed_event() {
        let json = r#"{"t":"fen","d":{"fen":"x","lm":"e2e4","wc":60,"bc":59}}"#;
        let event: LichessTvFeedEvent = serde_json::from_str(json).unwrap();
        match event {
            LichessTvFeedEvent::Fen(m) => assert_eq!(m.lm, "e2e4"),
            LichessTvFeedEvent::Featured(_) => panic!("expected fen"),
        }
    }
}
