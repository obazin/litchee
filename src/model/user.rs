//! User profile data shared across the account, users, and games concerns.

use serde::{Deserialize, Serialize};

use super::perf::LichessPerfs;
use super::title::LichessTitle;

/// A minimal user reference: just enough to identify and display a player.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LichessLightUser {
    /// The canonical (lowercased) user id.
    pub id: String,
    /// The display name.
    pub name: String,
    /// The player's flair, if set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flair: Option<String>,
    /// The player's title, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<LichessTitle>,
    /// Deprecated patron flag; prefer [`patron_color`](Self::patron_color).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patron: Option<bool>,
    /// The chosen Patron wing color; its presence marks an active patron.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patron_color: Option<u8>,
}

/// The free-form profile fields a user can fill in.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessProfile {
    /// Country/region flag code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flag: Option<String>,
    /// Free-text location.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    /// Biography.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    /// Real name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub real_name: Option<String>,
    /// Self-declared FIDE rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fide_rating: Option<u32>,
    /// Self-declared USCF rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uscf_rating: Option<u32>,
    /// Self-declared ECF rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ecf_rating: Option<u32>,
    /// Self-declared CFC rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cfc_rating: Option<u32>,
    /// Self-declared RCF rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rcf_rating: Option<u32>,
    /// Self-declared DSB rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dsb_rating: Option<u32>,
    /// Newline-separated social links.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub links: Option<String>,
}

/// Aggregate game counts for a user.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessCount {
    /// All games.
    pub all: u32,
    /// Rated games.
    pub rated: u32,
    /// Games against the computer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ai: Option<u32>,
    /// Drawn games.
    pub draw: u32,
    /// Drawn games that were rated.
    #[serde(rename = "drawH", default, skip_serializing_if = "Option::is_none")]
    pub draw_human: Option<u32>,
    /// Lost games.
    pub loss: u32,
    /// Lost games that were rated.
    #[serde(rename = "lossH", default, skip_serializing_if = "Option::is_none")]
    pub loss_human: Option<u32>,
    /// Won games.
    pub win: u32,
    /// Won games that were rated.
    #[serde(rename = "winH", default, skip_serializing_if = "Option::is_none")]
    pub win_human: Option<u32>,
    /// Bookmarked games.
    pub bookmark: u32,
    /// Games currently being played.
    pub playing: u32,
    /// Imported games.
    pub import: u32,
    /// Games against this account (only when querying another user).
    pub me: u32,
}

/// How long a user has spent playing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LichessPlayTime {
    /// Total seconds spent playing.
    pub total: u64,
    /// Seconds spent featured on Lichess TV.
    pub tv: u64,
}

/// A single streaming-platform channel link.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LichessStreamerChannel {
    /// The channel URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
}

/// A user's streamer channels.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessStreamerInfo {
    /// Twitch channel.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub twitch: Option<LichessStreamerChannel>,
    /// `YouTube` channel.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub youtube: Option<LichessStreamerChannel>,
}

/// A user profile as returned by most user-facing endpoints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessUser {
    /// The canonical (lowercased) user id.
    pub id: String,
    /// The display name.
    pub username: String,
    /// Per-perf rating statistics.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub perfs: Option<LichessPerfs>,
    /// The player's title, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<LichessTitle>,
    /// The player's flair, if set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flair: Option<String>,
    /// Account creation time (Unix milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    /// Present and `true` if the account is closed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    /// Present and `true` if the account is flagged for a TOS violation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tos_violation: Option<bool>,
    /// Free-form profile fields.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<LichessProfile>,
    /// Last-seen time (Unix milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seen_at: Option<i64>,
    /// Time spent playing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub play_time: Option<LichessPlayTime>,
    /// Deprecated patron flag; prefer [`patron_color`](Self::patron_color).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patron: Option<bool>,
    /// The chosen Patron wing color; its presence marks an active patron.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patron_color: Option<u8>,
    /// Whether the account is verified.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verified: Option<bool>,
}

/// An extended user profile, returned by the account and single-user endpoints.
///
/// Wraps [`LichessUser`] and adds the fields only present on the detailed view.
/// Several flags appear only for authenticated requests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessUserExtended {
    /// The wrapped core profile.
    #[serde(flatten)]
    pub user: LichessUser,
    /// Canonical profile URL.
    pub url: String,
    /// URL of the game the user is currently playing, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub playing: Option<String>,
    /// Aggregate game counts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count: Option<LichessCount>,
    /// Whether the user is currently streaming.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub streaming: Option<bool>,
    /// The user's streamer channels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub streamer: Option<LichessStreamerInfo>,
    /// Whether the authenticated user can follow this user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub followable: Option<bool>,
    /// Whether the authenticated user follows this user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub following: Option<bool>,
    /// Whether the authenticated user blocks this user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocking: Option<bool>,
    /// The user's FIDE id, if known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fide_id: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_extended_user_with_flattened_core() {
        let json = r#"{
            "id": "bobby", "username": "Bobby",
            "title": "GM", "createdAt": 1290415680000,
            "playTime": {"total": 1000, "tv": 20},
            "url": "https://lichess.org/@/Bobby",
            "count": {"all": 5, "rated": 3, "draw": 1, "loss": 1, "win": 3,
                      "bookmark": 0, "playing": 0, "import": 0, "me": 0},
            "followable": true, "following": false
        }"#;
        let user: LichessUserExtended = serde_json::from_str(json).unwrap();
        assert_eq!(user.user.id, "bobby");
        assert_eq!(user.user.title, Some(LichessTitle::Gm));
        assert_eq!(user.url, "https://lichess.org/@/Bobby");
        assert_eq!(user.count.unwrap().all, 5);
        assert_eq!(user.following, Some(false));
    }

    #[test]
    fn count_maps_human_suffixed_fields() {
        let json = r#"{"all":1,"rated":1,"draw":0,"drawH":0,"loss":0,"win":1,
                       "winH":1,"bookmark":0,"playing":0,"import":0,"me":0}"#;
        let count: LichessCount = serde_json::from_str(json).unwrap();
        assert_eq!(count.win_human, Some(1));
        assert_eq!(count.draw_human, Some(0));
    }
}
