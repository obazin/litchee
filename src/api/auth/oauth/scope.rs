//! `OAuth2` scopes.

use std::fmt;

/// An `OAuth2` scope that an access token may be granted.
///
/// Used when building an authorization URL to request specific permissions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Scope {
    /// Read preferences.
    PreferenceRead,
    /// Write preferences.
    PreferenceWrite,
    /// Read the account email address.
    EmailRead,
    /// Read external engines.
    EngineRead,
    /// Create, update, and delete external engines.
    EngineWrite,
    /// Read incoming challenges.
    ChallengeRead,
    /// Create, accept, and decline challenges.
    ChallengeWrite,
    /// Create, delete, and query bulk pairings.
    ChallengeBulk,
    /// Read private studies and broadcasts.
    StudyRead,
    /// Create, update, and delete studies and broadcasts.
    StudyWrite,
    /// Create tournaments.
    TournamentWrite,
    /// Create and join puzzle races.
    RacerWrite,
    /// Read puzzle activity.
    PuzzleRead,
    /// Write puzzle activity.
    PuzzleWrite,
    /// Read private team information.
    TeamRead,
    /// Join and leave teams.
    TeamWrite,
    /// Manage teams (kick members, send PMs).
    TeamLead,
    /// Read the list of followed players.
    FollowRead,
    /// Follow and unfollow other players.
    FollowWrite,
    /// Send private messages to other players.
    MsgWrite,
    /// Play with the Board API.
    BoardPlay,
    /// Play with the Bot API (bot accounts only).
    BotPlay,
    /// Use moderator tools, within the bounds of your permissions.
    WebMod,
}

impl Scope {
    /// Every scope, in the order Lichess documents them.
    pub const ALL: [Scope; 23] = [
        Scope::PreferenceRead,
        Scope::PreferenceWrite,
        Scope::EmailRead,
        Scope::EngineRead,
        Scope::EngineWrite,
        Scope::ChallengeRead,
        Scope::ChallengeWrite,
        Scope::ChallengeBulk,
        Scope::StudyRead,
        Scope::StudyWrite,
        Scope::TournamentWrite,
        Scope::RacerWrite,
        Scope::PuzzleRead,
        Scope::PuzzleWrite,
        Scope::TeamRead,
        Scope::TeamWrite,
        Scope::TeamLead,
        Scope::FollowRead,
        Scope::FollowWrite,
        Scope::MsgWrite,
        Scope::BoardPlay,
        Scope::BotPlay,
        Scope::WebMod,
    ];

    /// The wire representation, e.g. `"preference:read"`.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Scope::PreferenceRead => "preference:read",
            Scope::PreferenceWrite => "preference:write",
            Scope::EmailRead => "email:read",
            Scope::EngineRead => "engine:read",
            Scope::EngineWrite => "engine:write",
            Scope::ChallengeRead => "challenge:read",
            Scope::ChallengeWrite => "challenge:write",
            Scope::ChallengeBulk => "challenge:bulk",
            Scope::StudyRead => "study:read",
            Scope::StudyWrite => "study:write",
            Scope::TournamentWrite => "tournament:write",
            Scope::RacerWrite => "racer:write",
            Scope::PuzzleRead => "puzzle:read",
            Scope::PuzzleWrite => "puzzle:write",
            Scope::TeamRead => "team:read",
            Scope::TeamWrite => "team:write",
            Scope::TeamLead => "team:lead",
            Scope::FollowRead => "follow:read",
            Scope::FollowWrite => "follow:write",
            Scope::MsgWrite => "msg:write",
            Scope::BoardPlay => "board:play",
            Scope::BotPlay => "bot:play",
            Scope::WebMod => "web:mod",
        }
    }

    /// Parses a scope from its wire representation.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|scope| scope.as_str() == value)
    }
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_every_scope() {
        for scope in Scope::ALL {
            assert_eq!(Scope::parse(scope.as_str()), Some(scope));
        }
    }

    #[test]
    fn unknown_scope_is_none() {
        assert_eq!(Scope::parse("does:notexist"), None);
    }

    #[test]
    fn uses_colon_separated_wire_format() {
        assert_eq!(Scope::BoardPlay.as_str(), "board:play");
        assert_eq!(Scope::WebMod.to_string(), "web:mod");
    }
}
