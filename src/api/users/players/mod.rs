//! The Users API: look up players, their status, and head-to-head records.
//!
//! Reached through [`LichessClient::users`].

use std::collections::HashMap;

use reqwest::Method;
use reqwest::header::ACCEPT;
use serde::{Deserialize, Serialize};

use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;
use crate::model::{LichessLightUser, LichessTitle, LichessUser, LichessUserExtended};

mod activity;
mod perf_stat;

pub use activity::{
    LichessActivity, LichessActivityInterval, LichessActivityPuzzles, LichessActivityScore,
    LichessActivityScoreProgress, LichessActivityTournament, LichessActivityTournamentResult,
    LichessActivityTournaments,
};
pub use perf_stat::{
    LichessPerfStat, LichessPerfStatCount, LichessPerfStatDetails, LichessPerfStatPerf,
    LichessPerfStatPlayStreak, LichessPerfStatRatingAt, LichessPerfStatResult,
    LichessPerfStatResultStreak, LichessPerfStatResults, LichessPerfStatStreak,
    LichessPerfStatStreakSpan, LichessPerfStatStreakStamp,
};

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
    ///
    /// `query` toggles the optional extra sections (trophies, full profile,
    /// leaderboard rank, FIDE id); [`UserQuery::default`] requests none.
    pub async fn get(&self, username: &str, query: &UserQuery) -> Result<LichessUserExtended> {
        let path = format!("/api/user/{}", http::segment(username));
        let request = self
            .client
            .request(Method::GET, Host::Default, &path)
            .query(query);
        http::json(request, "LichessUserExtended").await
    }

    /// Gets several users by id (up to 300), returned in the requested order.
    ///
    /// `profile`/`rank` include each user's full profile / leaderboard rank.
    /// `POST /api/users`
    pub async fn get_many(
        &self,
        ids: &[&str],
        profile: Option<bool>,
        rank: Option<bool>,
    ) -> Result<Vec<LichessUser>> {
        let request = self
            .client
            .request(Method::POST, Host::Default, "/api/users")
            .query(&[("profile", profile), ("rank", rank)])
            .text_body(ids.join(","));
        http::json(request, "Vec<LichessUser>").await
    }

    /// Gets the real-time online/playing/streaming status of several users.
    ///
    /// The `with_*` flags add signal strength, current game ids, and game
    /// metadata. `GET /api/users/status`
    pub async fn statuses(
        &self,
        ids: &[&str],
        with_signal: Option<bool>,
        with_game_ids: Option<bool>,
        with_game_metas: Option<bool>,
    ) -> Result<Vec<LichessUserStatus>> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/users/status")
            .query(&[("ids", ids.join(","))])
            .query(&[
                ("withSignal", with_signal),
                ("withGameIds", with_game_ids),
                ("withGameMetas", with_game_metas),
            ]);
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
        let path = format!(
            "/api/crosstable/{}/{}",
            http::segment(user1),
            http::segment(user2)
        );
        let request = self
            .client
            .request(Method::GET, Host::Default, &path)
            .query(&[("matchup", matchup)]);
        http::json(request, "LichessCrosstable").await
    }

    /// Autocompletes usernames from a prefix (at least 3 characters).
    ///
    /// Returns a builder; refine it with the optional filters and finish with
    /// [`AutocompleteRequest::send`]. `GET /api/player/autocomplete`
    #[must_use]
    pub fn autocomplete(&self, term: &'a str) -> AutocompleteRequest<'a> {
        AutocompleteRequest::new(self.client, term)
    }

    /// Gets a user's rating history across all perfs.
    ///
    /// `GET /api/user/{username}/rating-history`
    pub async fn rating_history(&self, username: &str) -> Result<Vec<LichessRatingHistoryEntry>> {
        let path = format!("/api/user/{}/rating-history", http::segment(username));
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "Vec<LichessRatingHistoryEntry>").await
    }

    /// Gets a user's statistics in a single perf.
    ///
    /// `GET /api/user/{username}/perf/{perf}`
    pub async fn perf_stats(&self, username: &str, perf: &str) -> Result<LichessPerfStat> {
        let path = format!(
            "/api/user/{}/perf/{}",
            http::segment(username),
            http::segment(perf)
        );
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "LichessPerfStat").await
    }

    /// Gets a user's recent activity feed.
    ///
    /// `GET /api/user/{username}/activity`
    pub async fn activity(&self, username: &str) -> Result<Vec<LichessActivity>> {
        let path = format!("/api/user/{}/activity", http::segment(username));
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
        let path = format!("/api/player/top/{nb}/{}", http::segment(perf));
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
        let path = format!("/api/user/{}/note", http::segment(username));
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "Vec<LichessUserNote>").await
    }

    /// Writes a private note about a user. `POST /api/user/{username}/note`
    pub async fn write_note(&self, username: &str, text: &str) -> Result<()> {
        let path = format!("/api/user/{}/note", http::segment(username));
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .form(&[("text", text)]);
        http::ok(request).await
    }
}

/// Optional extra sections to include in a [`UsersApi::get`] lookup.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UserQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    trophies: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    profile: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rank: Option<bool>,
    #[serde(rename = "fideId", skip_serializing_if = "Option::is_none")]
    fide_id: Option<bool>,
}

impl UserQuery {
    /// Include the user's trophies.
    #[must_use]
    pub fn trophies(mut self, include: bool) -> Self {
        self.trophies = Some(include);
        self
    }

    /// Include the user's full profile.
    #[must_use]
    pub fn profile(mut self, include: bool) -> Self {
        self.profile = Some(include);
        self
    }

    /// Include the user's leaderboard rank.
    #[must_use]
    pub fn rank(mut self, include: bool) -> Self {
        self.rank = Some(include);
        self
    }

    /// Include the user's FIDE id.
    #[must_use]
    pub fn fide_id(mut self, include: bool) -> Self {
        self.fide_id = Some(include);
        self
    }
}

/// Query parameters for the username autocomplete.
#[derive(Debug, Default, Serialize)]
struct AutocompleteQuery<'a> {
    term: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    exists: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    object: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    names: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    friend: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    team: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tour: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    swiss: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    teacher: Option<bool>,
}

/// Builder for the username autocomplete (`GET /api/player/autocomplete`).
///
/// [`send`](Self::send) returns matching usernames. The spec's response-shape
/// variants are exposed as distinct terminals: [`exists`](Self::exists) returns
/// a bare boolean and [`objects`](Self::objects) returns full user objects.
#[derive(Debug)]
pub struct AutocompleteRequest<'a> {
    client: &'a LichessClient,
    query: AutocompleteQuery<'a>,
}

impl<'a> AutocompleteRequest<'a> {
    /// Creates the request builder for the search `term`.
    pub(crate) fn new(client: &'a LichessClient, term: &'a str) -> Self {
        Self {
            client,
            query: AutocompleteQuery {
                term,
                ..Default::default()
            },
        }
    }

    /// Return usernames with their preferred casing.
    #[must_use]
    pub fn names(mut self, value: bool) -> Self {
        self.query.names = Some(value);
        self
    }

    /// Prefer followed players (requires OAuth).
    #[must_use]
    pub fn friend(mut self, value: bool) -> Self {
        self.query.friend = Some(value);
        self
    }

    /// Restrict the search to a team (id/slug).
    #[must_use]
    pub fn team(mut self, team_id: &'a str) -> Self {
        self.query.team = Some(team_id);
        self
    }

    /// Restrict the search to an arena tournament (id).
    #[must_use]
    pub fn tour(mut self, tour_id: &'a str) -> Self {
        self.query.tour = Some(tour_id);
        self
    }

    /// Restrict the search to a swiss tournament (id).
    #[must_use]
    pub fn swiss(mut self, swiss_id: &'a str) -> Self {
        self.query.swiss = Some(swiss_id);
        self
    }

    /// Only return players who also have a teacher role.
    #[must_use]
    pub fn teacher(mut self, value: bool) -> Self {
        self.query.teacher = Some(value);
        self
    }

    /// Builds the GET request with the current query.
    fn request(&self) -> http::ApiRequest {
        self.client
            .request(Method::GET, Host::Default, "/api/player/autocomplete")
            .query(&self.query)
    }

    /// Executes the autocomplete, returning matching usernames.
    pub async fn send(self) -> Result<Vec<String>> {
        http::json(self.request(), "Vec<String>").await
    }

    /// Checks only whether a user with this exact term exists (`exists=true`).
    pub async fn exists(mut self) -> Result<bool> {
        self.query.exists = Some(true);
        http::json(self.request(), "bool").await
    }

    /// Executes the autocomplete, returning full user objects (`object=true`).
    pub async fn objects(mut self) -> Result<Vec<LichessLightUserOnline>> {
        self.query.object = Some(true);
        let response: AutocompleteObjectResponse =
            http::json(self.request(), "autocomplete result").await?;
        Ok(response.result)
    }
}

/// A user reference with online status, returned by the object-form autocomplete.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessLightUserOnline {
    /// The identifying/display fields.
    #[serde(flatten)]
    pub user: LichessLightUser,
    /// Whether the user is currently online.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub online: Option<bool>,
}

/// Wrapper for the `object=true` autocomplete response (`{ "result": [...] }`).
#[derive(Debug, Deserialize)]
struct AutocompleteObjectResponse {
    #[serde(default)]
    result: Vec<LichessLightUserOnline>,
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
    /// The identifying and display fields (id, name, title, flair, patron).
    #[serde(flatten)]
    pub user: LichessLightUser,
    /// Whether the user is currently online.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub online: Option<bool>,
    /// Whether the user is currently playing a game.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub playing: Option<bool>,
    /// Whether the user is currently streaming.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub streaming: Option<bool>,
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
    fn user_query_serializes_toggles() {
        assert_eq!(
            serde_urlencoded::to_string(UserQuery::default().trophies(true).fide_id(false))
                .unwrap(),
            "trophies=true&fideId=false"
        );
        assert_eq!(
            serde_urlencoded::to_string(UserQuery::default()).unwrap(),
            ""
        );
    }

    #[test]
    fn autocomplete_query_serializes_filters() {
        let query = AutocompleteQuery {
            term: "bob",
            names: Some(true),
            team: Some("coders"),
            ..Default::default()
        };
        let encoded = serde_urlencoded::to_string(&query).unwrap();
        assert_eq!(encoded, "term=bob&names=true&team=coders");
    }

    #[test]
    fn parses_user_status_flags() {
        let json = r#"{"id":"bobby","name":"Bobby","online":true,"playing":false}"#;
        let status: LichessUserStatus = serde_json::from_str(json).unwrap();
        assert_eq!(status.user.id, "bobby");
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
