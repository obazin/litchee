//! DTOs for the Games API: exported/streamed games and their nested types.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::model::{LichessColor, LichessLightUser, LichessSpeed, LichessVariantKey};

/// The terminal (or current) status of a game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum LichessGameStatusName {
    /// The game has been created but not started.
    Created,
    /// The game is in progress.
    Started,
    /// The game was aborted.
    Aborted,
    /// Won by checkmate.
    Mate,
    /// A player resigned.
    Resign,
    /// Drawn by stalemate.
    Stalemate,
    /// A player's time ran out (flagged).
    Timeout,
    /// Drawn.
    Draw,
    /// A player ran out of time.
    Outoftime,
    /// A player was caught cheating.
    Cheat,
    /// A player did not start in time.
    NoStart,
    /// The game ended in an unknown way.
    UnknownFinish,
    /// A draw was claimed by insufficient material.
    InsufficientMaterialClaim,
    /// The game ended by a variant-specific rule.
    VariantEnd,
}

/// The opening of a game.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessGameOpening {
    /// The ECO code (e.g. `B01`).
    pub eco: String,
    /// The opening name.
    pub name: String,
    /// The ply at which the opening is identified.
    pub ply: u32,
}

/// A judgment annotation on an analysed move.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessMoveJudgment {
    /// The severity (`Inaccuracy`, `Mistake`, or `Blunder`).
    pub name: String,
    /// The human-readable comment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

/// Per-move engine analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessGameMoveAnalysis {
    /// Evaluation in centipawns.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eval: Option<i32>,
    /// Moves until forced mate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mate: Option<i32>,
    /// Best move in UCI notation (only if the played move was inaccurate).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub best: Option<String>,
    /// Best variation in SAN notation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variation: Option<String>,
    /// Judgment annotation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub judgment: Option<LichessMoveJudgment>,
}

/// Aggregate analysis statistics for one player in a game.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessPlayerAnalysis {
    /// Number of inaccuracies.
    pub inaccuracy: u32,
    /// Number of mistakes.
    pub mistake: u32,
    /// Number of blunders.
    pub blunder: u32,
    /// Average centipawn loss.
    pub acpl: u32,
    /// Accuracy percentage, when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accuracy: Option<u32>,
}

/// One side of a game.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessGamePlayer {
    /// The player's light user info (absent for AI players).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<LichessLightUser>,
    /// The player's rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rating: Option<u32>,
    /// The rating change from this game.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rating_diff: Option<i32>,
    /// The player's name (for anonymous or AI players).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Whether the rating is provisional.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provisional: Option<bool>,
    /// The AI level, for games against the computer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ai_level: Option<u32>,
    /// Aggregate analysis for this player.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub analysis: Option<LichessPlayerAnalysis>,
    /// The player's team id, in team games.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team: Option<String>,
}

/// Both sides of a game.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessGamePlayers {
    /// The white player.
    pub white: LichessGamePlayer,
    /// The black player.
    pub black: LichessGamePlayer,
}

/// A game's clock configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessGameClock {
    /// Initial time in seconds.
    pub initial: u32,
    /// Increment per move in seconds.
    pub increment: u32,
    /// Total estimated time in seconds.
    pub total_time: u32,
}

/// The ply boundaries of a game's phases.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessGameDivision {
    /// Ply at which the middlegame begins.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub middle: Option<u32>,
    /// Ply at which the endgame begins.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<u32>,
}

/// The arena tournament a [`LichessGame`] is from.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessGameArenaTour {
    /// The arena tournament id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The arena tournament name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// The swiss tournament a [`LichessGame`] is from.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessGameSwissTour {
    /// The swiss tournament id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// A game, as returned in JSON by the export and stream endpoints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessGame {
    /// The game id.
    pub id: String,
    /// Whether the game is rated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rated: Option<bool>,
    /// The variant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<LichessVariantKey>,
    /// The speed category.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed: Option<LichessSpeed>,
    /// The perf key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub perf: Option<String>,
    /// Creation time (Unix milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    /// Last-move time (Unix milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_move_at: Option<i64>,
    /// The game status.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<LichessGameStatusName>,
    /// The game source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// The players.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub players: Option<LichessGamePlayers>,
    /// The initial FEN, for games not started from the standard position.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_fen: Option<String>,
    /// The winner's color, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub winner: Option<LichessColor>,
    /// The opening.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opening: Option<LichessGameOpening>,
    /// The moves in UCI or SAN, space-separated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub moves: Option<String>,
    /// The full PGN, when requested with `pgnInJson`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pgn: Option<String>,
    /// Days per turn, for correspondence games.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub days_per_turn: Option<u32>,
    /// Per-move analysis, when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub analysis: Option<Vec<LichessGameMoveAnalysis>>,
    /// The arena tournament the game is from, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arena_tour: Option<LichessGameArenaTour>,
    /// The swiss tournament the game is from, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub swiss_tour: Option<LichessGameSwissTour>,
    /// The clock configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clock: Option<LichessGameClock>,
    /// Per-move clock readings, in centiseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clocks: Option<Vec<u32>>,
    /// The phase boundaries.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub division: Option<LichessGameDivision>,
}

/// The result of importing a game.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessImportedGame {
    /// The id of the imported game.
    pub id: String,
    /// The URL of the imported game.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// The opponent in a [`LichessNowPlayingGame`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessNowPlayingOpponent {
    /// The opponent's id.
    pub id: String,
    /// The opponent's username.
    pub username: String,
    /// The opponent's rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rating: Option<u32>,
    /// The opponent's rating change.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rating_diff: Option<i32>,
    /// The AI level, if playing the computer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ai: Option<u32>,
}

/// A game the authenticated user is currently playing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessNowPlayingGame {
    /// The game id.
    pub game_id: String,
    /// The full game id (includes the player token).
    pub full_id: String,
    /// The authenticated user's color.
    pub color: LichessColor,
    /// The current position (FEN).
    pub fen: String,
    /// Whether the user has moved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_moved: Option<bool>,
    /// Whether it is the user's turn.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_my_turn: Option<bool>,
    /// The last move in UCI.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_move: Option<String>,
    /// The opponent.
    pub opponent: LichessNowPlayingOpponent,
    /// The perf key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub perf: Option<String>,
    /// Whether the game is rated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rated: Option<bool>,
    /// Seconds left on the user's clock.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seconds_left: Option<i64>,
    /// The game source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// The speed category.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed: Option<LichessSpeed>,
    /// The variant key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<LichessVariantKey>,
    /// The arena tournament id, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tournament_id: Option<String>,
    /// The swiss tournament id, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub swiss_id: Option<String>,
}

/// The games the authenticated user is currently playing.
/// `GET /api/account/playing`
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessNowPlaying {
    /// Number of games where it is the user's turn to play.
    pub nb_my_turn: u32,
    /// The ongoing games.
    #[serde(default)]
    pub now_playing: Vec<LichessNowPlayingGame>,
}

/// A spectator chat message on a game.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessGameChatMessage {
    /// The sender's username.
    pub user: String,
    /// The message text.
    pub text: String,
}

/// A move-by-move update from a game move stream. `GET /api/stream/game/{id}`
///
/// The first message is the full game; later messages carry the latest move
/// and clocks. Typed common fields plus a lossless `other` map.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessGameMoveUpdate {
    /// The current position (FEN).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fen: Option<String>,
    /// The last move in UCI.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lm: Option<String>,
    /// White's clock in centiseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wc: Option<i64>,
    /// Black's clock in centiseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bc: Option<i64>,
    /// Any other fields (e.g. the full first message).
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_game() {
        let json = r#"{
            "id":"q7ZvsdUF","rated":true,"variant":"standard","speed":"blitz",
            "perf":"blitz","createdAt":1514505150384,"status":"mate",
            "winner":"white",
            "players":{
                "white":{"user":{"id":"a","name":"A"},"rating":1600,"ratingDiff":8},
                "black":{"user":{"id":"b","name":"B"},"rating":1590,"ratingDiff":-8}
            },
            "opening":{"eco":"B01","name":"Scandinavian","ply":2},
            "moves":"e4 d5 exd5",
            "clock":{"initial":300,"increment":3,"totalTime":420}
        }"#;
        let game: LichessGame = serde_json::from_str(json).unwrap();
        assert_eq!(game.id, "q7ZvsdUF");
        assert_eq!(game.status, Some(LichessGameStatusName::Mate));
        assert_eq!(game.winner, Some(LichessColor::White));
        assert_eq!(game.players.unwrap().white.rating_diff, Some(8));
        assert_eq!(game.clock.unwrap().total_time, 420);
    }

    #[test]
    fn parses_minimal_game() {
        let game: LichessGame = serde_json::from_str(r#"{"id":"abcd1234"}"#).unwrap();
        assert_eq!(game.id, "abcd1234");
        assert!(game.players.is_none());
    }

    #[test]
    fn status_uses_camel_case_names() {
        let game: LichessGame =
            serde_json::from_str(r#"{"id":"x","status":"variantEnd"}"#).unwrap();
        assert_eq!(game.status, Some(LichessGameStatusName::VariantEnd));
    }

    #[test]
    fn parses_arena_and_swiss_tour_objects() {
        let json = r#"{"id":"g","arenaTour":{"id":"abc","name":"Hourly"},
            "swissTour":{"id":"xyz"}}"#;
        let game: LichessGame = serde_json::from_str(json).unwrap();
        let arena = game.arena_tour.unwrap();
        assert_eq!(arena.id.as_deref(), Some("abc"));
        assert_eq!(arena.name.as_deref(), Some("Hourly"));
        assert_eq!(game.swiss_tour.unwrap().id.as_deref(), Some("xyz"));
    }
}
