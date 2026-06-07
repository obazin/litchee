//! The Users API: look up players, their status, and head-to-head records.
//!
//! Reached through [`LichessClient::users`].

use std::collections::HashMap;

use reqwest::Method;
use reqwest::header::{ACCEPT, CONTENT_TYPE};

use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;
use crate::model::{LichessUser, LichessUserExtended};

mod model;

pub use model::{
    LichessActivity, LichessActivityInterval, LichessCrosstable, LichessGlicko, LichessLeaderboard,
    LichessLiveStreamer, LichessMatchup, LichessPerfStat, LichessPerfStatPerf,
    LichessRatingHistoryEntry, LichessStreamDetails, LichessTopUser, LichessTopUserPerf,
    LichessUserNote, LichessUserStatus,
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
