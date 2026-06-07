//! Events streamed from a board/bot game (`gameFull`, `gameState`, …).
//!
//! These types are shared by the Board and Bot APIs.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::gameplay::challenges::LichessChallenge;
use crate::api::gameplay::games::LichessGameStatusName;
use crate::model::{LichessColor, LichessSpeed, LichessTitle, LichessVariantKey};

/// A chat room within a game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum LichessChatRoom {
    /// The private chat between the two players.
    Player,
    /// The public spectator chat.
    Spectator,
}

/// A player as described in a game-stream event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessGameEventPlayer {
    /// The player's id (absent for AI players).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The player's display name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The AI level, for computer players.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ai_level: Option<u32>,
    /// The player's title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<LichessTitle>,
    /// The player's rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rating: Option<u32>,
    /// Whether the rating is provisional.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provisional: Option<bool>,
}

/// Variant details in a game-stream event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessVariantInfo {
    /// The variant key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<LichessVariantKey>,
    /// The full variant name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// A short variant name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub short: Option<String>,
}

/// Clock configuration in a game-stream event (milliseconds).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessGameEventClock {
    /// Initial time in milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial: Option<i64>,
    /// Increment in milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub increment: Option<i64>,
}

/// First-move expiration info.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessGameExpiration {
    /// Milliseconds since the last move or game start.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idle_millis: Option<i64>,
    /// Milliseconds each player has for their first move.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub millis_to_move: Option<i64>,
}

/// The current state of a game (mutable part of the stream).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessGameState {
    /// All moves so far, in UCI, space-separated.
    pub moves: String,
    /// White's remaining time in milliseconds.
    pub wtime: i64,
    /// Black's remaining time in milliseconds.
    pub btime: i64,
    /// White's increment in milliseconds.
    pub winc: i64,
    /// Black's increment in milliseconds.
    pub binc: i64,
    /// The game status.
    pub status: LichessGameStatusName,
    /// The winner's color, if the game is over.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub winner: Option<LichessColor>,
    /// Whether White is offering a draw.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wdraw: Option<bool>,
    /// Whether Black is offering a draw.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bdraw: Option<bool>,
    /// Whether White is proposing a takeback.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wtakeback: Option<bool>,
    /// Whether Black is proposing a takeback.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub btakeback: Option<bool>,
    /// First-move expiration info.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expiration: Option<LichessGameExpiration>,
}

/// The full game data (immutable part), sent first in the stream.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessGameFull {
    /// The game id.
    pub id: String,
    /// The variant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<LichessVariantInfo>,
    /// The clock configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clock: Option<LichessGameEventClock>,
    /// Whether the game is rated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rated: Option<bool>,
    /// Creation time (Unix milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    /// The white player.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub white: Option<LichessGameEventPlayer>,
    /// The black player.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub black: Option<LichessGameEventPlayer>,
    /// The initial FEN (`"startpos"` for standard starts).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_fen: Option<String>,
    /// The current game state.
    pub state: LichessGameState,
    /// Days per turn, for correspondence games.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub days_per_turn: Option<u32>,
    /// The arena tournament id, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tournament_id: Option<String>,
}

/// A chat message in a game stream.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessChatLine {
    /// The room the message was sent in.
    pub room: LichessChatRoom,
    /// The sender's username.
    pub username: String,
    /// The message text.
    pub text: String,
}

/// Notification that the opponent has left (or returned).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessOpponentGone {
    /// Whether the opponent is currently gone.
    pub gone: bool,
    /// Seconds until a win can be claimed, if counting down.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claim_win_in_seconds: Option<u32>,
}

/// An event from a board/bot game stream.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum LichessBoardEvent {
    /// Full game data; always the first message.
    ///
    /// Boxed because it is much larger than the other variants.
    #[serde(rename = "gameFull")]
    GameFull(Box<LichessGameFull>),
    /// Current game state (sent on each move and on draw/takeback/end).
    #[serde(rename = "gameState")]
    GameState(LichessGameState),
    /// A chat message.
    #[serde(rename = "chatLine")]
    ChatLine(LichessChatLine),
    /// The opponent left or returned.
    #[serde(rename = "opponentGone")]
    OpponentGone(LichessOpponentGone),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_game_full_with_nested_state() {
        let json = r#"{"type":"gameFull","id":"g","rated":false,
            "white":{"id":"a","name":"A","rating":1700},
            "black":{"id":"b","name":"B","rating":1600},
            "initialFen":"startpos",
            "state":{"type":"gameState","moves":"e2e4","wtime":900000,"btime":900000,
                     "winc":0,"binc":0,"status":"started"}}"#;
        let event: LichessBoardEvent = serde_json::from_str(json).unwrap();
        match event {
            LichessBoardEvent::GameFull(full) => {
                assert_eq!(full.id, "g");
                assert_eq!(full.state.moves, "e2e4");
            }
            other => panic!("expected gameFull, got {other:?}"),
        }
    }

    #[test]
    fn parses_game_state_and_chat_events() {
        let state: LichessBoardEvent = serde_json::from_str(
            r#"{"type":"gameState","moves":"e2e4 e7e5","wtime":1,"btime":2,
                "winc":0,"binc":0,"status":"started"}"#,
        )
        .unwrap();
        assert!(matches!(state, LichessBoardEvent::GameState(_)));

        let chat: LichessBoardEvent = serde_json::from_str(
            r#"{"type":"chatLine","room":"player","username":"a","text":"hi"}"#,
        )
        .unwrap();
        match chat {
            LichessBoardEvent::ChatLine(line) => assert_eq!(line.room, LichessChatRoom::Player),
            other => panic!("expected chatLine, got {other:?}"),
        }
    }
}

/// Summary info about a game referenced by an incoming event.
///
/// Typed common fields plus a lossless `other` map.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessGameEventInfo {
    /// The game id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The game id (duplicate of `id` in some events).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game_id: Option<String>,
    /// The full game id (includes the player token).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_id: Option<String>,
    /// The authenticated user's color.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<LichessColor>,
    /// The current position (FEN).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fen: Option<String>,
    /// The last move in UCI.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_move: Option<String>,
    /// The game source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// The speed category.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed: Option<LichessSpeed>,
    /// Whether it is the user's turn.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_my_turn: Option<bool>,
    /// Any other fields.
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

/// An event from the global incoming-event stream. `GET /api/stream/event`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum LichessIncomingEvent {
    /// A game started.
    #[serde(rename = "gameStart")]
    GameStart {
        /// The started game.
        game: LichessGameEventInfo,
    },
    /// A game finished.
    #[serde(rename = "gameFinish")]
    GameFinish {
        /// The finished game.
        game: LichessGameEventInfo,
    },
    /// A challenge was received or sent.
    #[serde(rename = "challenge")]
    Challenge {
        /// The challenge (boxed: much larger than the other variants).
        challenge: Box<LichessChallenge>,
    },
    /// A challenge was canceled.
    #[serde(rename = "challengeCanceled")]
    ChallengeCanceled {
        /// The canceled challenge.
        challenge: Box<LichessChallenge>,
    },
    /// A challenge was declined.
    #[serde(rename = "challengeDeclined")]
    ChallengeDeclined {
        /// The declined challenge.
        challenge: Box<LichessChallenge>,
    },
}

#[cfg(test)]
mod incoming_tests {
    use super::*;

    #[test]
    fn parses_game_start_event() {
        let json = r#"{"type":"gameStart","game":{"gameId":"g","color":"white","fen":"x"}}"#;
        let event: LichessIncomingEvent = serde_json::from_str(json).unwrap();
        match event {
            LichessIncomingEvent::GameStart { game } => {
                assert_eq!(game.game_id.as_deref(), Some("g"));
            }
            other => panic!("expected gameStart, got {other:?}"),
        }
    }

    #[test]
    fn parses_challenge_event() {
        let json = r#"{"type":"challenge","challenge":{"id":"c","url":"u","status":"created"}}"#;
        let event: LichessIncomingEvent = serde_json::from_str(json).unwrap();
        assert!(matches!(event, LichessIncomingEvent::Challenge { .. }));
    }
}
