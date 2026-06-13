//! The Users API: look up players, their status, and head-to-head records.
//!
//! Reached through [`LichessClient::users`].

use std::collections::HashMap;

use reqwest::Method;
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;
use crate::model::{LichessLightUser, LichessTitle, LichessUser, LichessUserExtended};

/// Accessor for the Users API.
#[derive(Debug)]
pub struct UsersApi<'a> {
    client: &'a LichessClient,
}

impl<'a> UsersApi<'a> {
    /// Binds the accessor to a client.
    pub(crate) fn new(client: &'a LichessClient) -> Self {
        Self { client }
    }

    /// Gets the extended profile of a single user.
    ///
    /// `GET /api/user/{username}`
    pub async fn get(&self, username: &str) -> Result<LichessUserExtended> {
        let path = format!("/api/user/{username}");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "LichessUserExtended").await
    }

    /// Gets several users by id (up to 300), returned in the requested order.
    ///
    /// `POST /api/users`
    pub async fn get_many(&self, ids: &[&str]) -> Result<Vec<LichessUser>> {
        let request = self
            .client
            .request(Method::POST, Host::Default, "/api/users")
            .header(CONTENT_TYPE, "text/plain")
            .body(ids.join(","));
        http::json(request, "Vec<LichessUser>").await
    }

    /// Gets the real-time online/playing/streaming status of several users.
    ///
    /// `GET /api/users/status`
    pub async fn statuses(&self, ids: &[&str]) -> Result<Vec<LichessUserStatus>> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/users/status")
            .query(&[("ids", ids.join(","))]);
        http::json(request, "Vec<LichessUserStatus>").await
    }

    /// Gets the head-to-head record of two users.
    ///
    /// When `matchup` is `true` and the players are currently facing off, the
    /// current-match score is also returned. `GET /api/crosstable/{u1}/{u2}`
    pub async fn crosstable(
        &self,
        user1: &str,
        user2: &str,
        matchup: bool,
    ) -> Result<LichessCrosstable> {
        let path = format!("/api/crosstable/{user1}/{user2}");
        let request = self
            .client
            .request(Method::GET, Host::Default, &path)
            .query(&[("matchup", matchup)]);
        http::json(request, "LichessCrosstable").await
    }

    /// Autocompletes usernames from a prefix (at least 3 characters).
    ///
    /// `GET /api/player/autocomplete`
    pub async fn autocomplete(&self, term: &str) -> Result<Vec<String>> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/player/autocomplete")
            .query(&[("term", term)]);
        http::json(request, "Vec<String>").await
    }

    /// Gets a user's rating history across all perfs.
    ///
    /// `GET /api/user/{username}/rating-history`
    pub async fn rating_history(&self, username: &str) -> Result<Vec<LichessRatingHistoryEntry>> {
        let path = format!("/api/user/{username}/rating-history");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "Vec<LichessRatingHistoryEntry>").await
    }

    /// Gets a user's statistics in a single perf.
    ///
    /// `GET /api/user/{username}/perf/{perf}`
    pub async fn perf_stats(&self, username: &str, perf: &str) -> Result<LichessPerfStat> {
        let path = format!("/api/user/{username}/perf/{perf}");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "LichessPerfStat").await
    }

    /// Gets a user's recent activity feed.
    ///
    /// `GET /api/user/{username}/activity`
    pub async fn activity(&self, username: &str) -> Result<Vec<LichessActivity>> {
        let path = format!("/api/user/{username}/activity");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "Vec<LichessActivity>").await
    }

    /// Gets the top-10 players for every standard perf. `GET /api/player`
    pub async fn leaderboards(&self) -> Result<HashMap<String, Vec<LichessTopUser>>> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/player");
        http::json(request, "leaderboards").await
    }

    /// Gets the top `nb` players for a single perf.
    ///
    /// `GET /api/player/top/{nb}/{perfType}`
    pub async fn top(&self, perf: &str, nb: u32) -> Result<LichessLeaderboard> {
        let path = format!("/api/player/top/{nb}/{perf}");
        let request = self
            .client
            .request(Method::GET, Host::Default, &path)
            .header(ACCEPT, "application/vnd.lichess.v3+json");
        http::json(request, "LichessLeaderboard").await
    }

    /// Lists the currently-live streamers. `GET /api/streamer/live`
    pub async fn live_streamers(&self) -> Result<Vec<LichessLiveStreamer>> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/streamer/live");
        http::json(request, "Vec<LichessLiveStreamer>").await
    }

    /// Reads the private notes about a user. `GET /api/user/{username}/note`
    pub async fn notes(&self, username: &str) -> Result<Vec<LichessUserNote>> {
        let path = format!("/api/user/{username}/note");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "Vec<LichessUserNote>").await
    }

    /// Writes a private note about a user. `POST /api/user/{username}/note`
    pub async fn write_note(&self, username: &str, text: &str) -> Result<()> {
        let path = format!("/api/user/{username}/note");
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .form(&[("text", text)]);
        http::ok(request).await
    }
}

impl LichessClient {
    /// Users API: look up players, their status, and head-to-head records.
    #[must_use]
    pub fn users(&self) -> UsersApi<'_> {
        UsersApi::new(self)
    }
}

/// The real-time status of a user: online / playing / streaming flags.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessUserStatus {
    /// The canonical (lowercased) user id.
    pub id: String,
    /// The display name.
    pub name: String,
    /// The player's title, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<LichessTitle>,
    /// The player's flair, if set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flair: Option<String>,
    /// Whether the user is currently online.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub online: Option<bool>,
    /// Whether the user is currently playing a game.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub playing: Option<bool>,
    /// Whether the user is currently streaming.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub streaming: Option<bool>,
    /// Deprecated patron flag; prefer [`patron_color`](Self::patron_color).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patron: Option<bool>,
    /// The chosen Patron wing color; its presence marks an active patron.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patron_color: Option<u8>,
    /// Network signal strength 1–4, only when requested with `withSignal`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signal: Option<u8>,
    /// Id of the game being played, only when requested with `withGameIds`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub playing_id: Option<String>,
}

/// Head-to-head totals between two players.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessCrosstable {
    /// Each user's cumulative score (half-points), keyed by user id.
    pub users: HashMap<String, f64>,
    /// Total number of games played between the two users.
    pub nb_games: u32,
    /// Current-match data, present only when `matchup` was requested and the
    /// two users are playing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matchup: Option<LichessMatchup>,
}

/// The ongoing match score between two players.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LichessMatchup {
    /// Each user's score in the current match, keyed by user id.
    pub users: HashMap<String, f64>,
    /// Number of games in the current match.
    pub nb_games: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_user_status_flags() {
        let json = r#"{"id":"bobby","name":"Bobby","online":true,"playing":false}"#;
        let status: LichessUserStatus = serde_json::from_str(json).unwrap();
        assert_eq!(status.id, "bobby");
        assert_eq!(status.online, Some(true));
        assert_eq!(status.playing, Some(false));
        assert_eq!(status.streaming, None);
    }

    #[test]
    fn parses_crosstable_scores() {
        let json = r#"{"users":{"neio":201.5,"thibault":144.5},"nbGames":346}"#;
        let crosstable: LichessCrosstable = serde_json::from_str(json).unwrap();
        assert_eq!(crosstable.nb_games, 346);
        assert_eq!(crosstable.users.get("neio"), Some(&201.5));
        assert!(crosstable.matchup.is_none());
    }
}

/// An entry in a user's rating history for one perf.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessRatingHistoryEntry {
    /// The perf name (e.g. `"Blitz"`).
    pub name: String,
    /// Data points, each `[year, month, day, rating]` (month is 0-indexed).
    #[serde(default)]
    pub points: Vec<[i32; 4]>,
}

/// A private note about another player.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessUserNote {
    /// The author of the note.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<LichessLightUser>,
    /// The user the note is about.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<LichessLightUser>,
    /// The note text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// When the note was written (Unix milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<i64>,
}

/// A perf rating/progress pair within a [`LichessTopUser`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessTopUserPerf {
    /// The rating.
    pub rating: i32,
    /// The recent progress.
    pub progress: i32,
}

/// A leaderboard player.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessTopUser {
    /// The user id.
    pub id: String,
    /// The display name.
    pub username: String,
    /// Per-perf rating and progress.
    #[serde(default)]
    pub perfs: HashMap<String, LichessTopUserPerf>,
    /// The player's title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<LichessTitle>,
    /// The chosen Patron wing color.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patron_color: Option<u8>,
    /// Whether the user is online.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub online: Option<bool>,
}

/// A single-perf leaderboard. `GET /api/player/top/{nb}/{perfType}`
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessLeaderboard {
    /// The top players.
    #[serde(default)]
    pub users: Vec<LichessTopUser>,
}

/// Glicko-2 rating details.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessGlicko {
    /// The rating.
    pub rating: f64,
    /// The rating deviation.
    pub deviation: f64,
    /// Whether the rating is provisional.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provisional: Option<bool>,
}

/// The rating part of a [`LichessPerfStat`].
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessPerfStatPerf {
    /// The Glicko-2 rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub glicko: Option<LichessGlicko>,
    /// Number of games played.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nb: Option<u32>,
    /// Recent progress.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progress: Option<i32>,
}

/// Statistics for one of a user's perfs. `GET /api/user/{username}/perf/{perf}`
///
/// Models the headline fields; the detailed `stat` aggregate is not decoded.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessPerfStat {
    /// The user's percentile within this perf.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub percentile: Option<f64>,
    /// The user's rank within this perf.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rank: Option<u32>,
    /// The user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<LichessLightUser>,
    /// The rating details.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub perf: Option<LichessPerfStatPerf>,
}

/// The time range of a [`LichessActivity`] entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessActivityInterval {
    /// Start time (Unix milliseconds).
    pub start: i64,
    /// End time (Unix milliseconds).
    pub end: i64,
}

/// One day of a user's activity. `GET /api/user/{username}/activity`
///
/// Models the time interval; the per-category activity payloads vary widely and
/// are not all decoded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessActivity {
    /// The time range this entry covers.
    pub interval: LichessActivityInterval,
}

/// The stream details of a [`LichessLiveStreamer`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessStreamDetails {
    /// The streaming service (`twitch` or `youtube`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    /// The stream title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// The stream language.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// A currently-live streamer. `GET /api/streamer/live`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessLiveStreamer {
    /// The streamer's light user info.
    #[serde(flatten)]
    pub user: LichessLightUser,
    /// The current stream details.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream: Option<LichessStreamDetails>,
}

#[cfg(test)]
mod added_tests {
    use super::*;

    #[test]
    fn parses_rating_history() {
        let json = r#"[{"name":"Bullet","points":[[2011,0,8,1472],[2011,8,12,1314]]}]"#;
        let history: Vec<LichessRatingHistoryEntry> = serde_json::from_str(json).unwrap();
        assert_eq!(history[0].name, "Bullet");
        assert_eq!(history[0].points[1], [2011, 8, 12, 1314]);
    }

    #[test]
    fn parses_leaderboard_top_user() {
        let json = r#"{"users":[{"id":"a","username":"A",
            "perfs":{"bullet":{"rating":2900,"progress":5}},"title":"GM"}]}"#;
        let board: LichessLeaderboard = serde_json::from_str(json).unwrap();
        assert_eq!(board.users[0].perfs["bullet"].rating, 2900);
        assert_eq!(board.users[0].title, Some(LichessTitle::Gm));
    }

    #[test]
    fn parses_live_streamer_with_flattened_user() {
        let json = r#"{"id":"a","name":"A","stream":{"service":"twitch","status":"Live!"}}"#;
        let streamer: LichessLiveStreamer = serde_json::from_str(json).unwrap();
        assert_eq!(streamer.user.id, "a");
        assert_eq!(streamer.stream.unwrap().service.as_deref(), Some("twitch"));
    }
}
