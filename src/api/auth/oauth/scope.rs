//! `OAuth2` scopes.

use std::fmt;

/// Declares every scope once, as `Variant => "wire:format"` pairs, and derives
/// the enum, [`Scope::ALL`], and [`Scope::as_str`] from that single table.
///
/// Keeping the three in one place means a new scope cannot be half-added: the
/// generated `as_str` is an exhaustive `match`, so a variant without a wire
/// string fails to compile, and `ALL`'s declared length has to be updated in
/// step with the table.
macro_rules! scopes {
    ($count:literal: $( $(#[$meta:meta])* $variant:ident => $wire:literal, )+) => {
        /// An `OAuth2` scope that an access token may be granted.
        ///
        /// Used when building an authorization URL to request specific
        /// permissions.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[non_exhaustive]
        pub enum Scope {
            $( $(#[$meta])* $variant, )+
        }

        impl Scope {
            /// Every scope, in the order Lichess documents them.
            pub const ALL: [Scope; $count] = [ $( Scope::$variant, )+ ];

            /// The wire representation, e.g. `"preference:read"`.
            #[must_use]
            pub fn as_str(self) -> &'static str {
                match self {
                    $( Scope::$variant => $wire, )+
                }
            }
        }
    };
}

scopes! { 23:
    /// Read preferences.
    PreferenceRead => "preference:read",
    /// Write preferences.
    PreferenceWrite => "preference:write",
    /// Read the account email address.
    EmailRead => "email:read",
    /// Read external engines.
    EngineRead => "engine:read",
    /// Create, update, and delete external engines.
    EngineWrite => "engine:write",
    /// Read incoming challenges.
    ChallengeRead => "challenge:read",
    /// Create, accept, and decline challenges.
    ChallengeWrite => "challenge:write",
    /// Create, delete, and query bulk pairings.
    ChallengeBulk => "challenge:bulk",
    /// Read private studies and broadcasts.
    StudyRead => "study:read",
    /// Create, update, and delete studies and broadcasts.
    StudyWrite => "study:write",
    /// Create tournaments.
    TournamentWrite => "tournament:write",
    /// Create and join puzzle races.
    RacerWrite => "racer:write",
    /// Read puzzle activity.
    PuzzleRead => "puzzle:read",
    /// Write puzzle activity.
    PuzzleWrite => "puzzle:write",
    /// Read private team information.
    TeamRead => "team:read",
    /// Join and leave teams.
    TeamWrite => "team:write",
    /// Manage teams (kick members, send PMs).
    TeamLead => "team:lead",
    /// Read the list of followed players.
    FollowRead => "follow:read",
    /// Follow and unfollow other players.
    FollowWrite => "follow:write",
    /// Send private messages to other players.
    MsgWrite => "msg:write",
    /// Play with the Board API.
    BoardPlay => "board:play",
    /// Play with the Bot API (bot accounts only).
    BotPlay => "bot:play",
    /// Use moderator tools, within the bounds of your permissions.
    WebMod => "web:mod",
}

impl Scope {
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

    #[test]
    fn wire_strings_are_unique() {
        let mut wires: Vec<&str> = Scope::ALL.iter().map(|scope| scope.as_str()).collect();
        wires.sort_unstable();
        let total = wires.len();
        wires.dedup();
        assert_eq!(wires.len(), total, "duplicate wire strings in the table");
    }
}
