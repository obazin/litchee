//! The Broadcasts API: official broadcasts, rounds, players, and PGN.
//!
//! Reached through [`LichessClient::broadcasts`].

use std::collections::HashMap;

use futures_util::stream::BoxStream;
use reqwest::Method;
use reqwest::header::CONTENT_TYPE;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;

/// Accessor for the Broadcasts API.
#[derive(Debug)]
pub struct BroadcastsApi<'a> {
    client: &'a LichessClient,
}

impl<'a> BroadcastsApi<'a> {
    /// Binds the accessor to a client.
    pub(crate) fn new(client: &'a LichessClient) -> Self {
        Self { client }
    }

    /// Streams official broadcasts. `GET /api/broadcast`
    pub async fn official(&self) -> Result<BoxStream<'static, Result<LichessBroadcast>>> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/broadcast");
        http::stream(request, self.client.max_line_bytes()).await
    }

    /// Gets the top broadcasts (active, upcoming, past). `GET /api/broadcast/top`
    pub async fn top(&self) -> Result<LichessBroadcastTop> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/broadcast/top");
        http::json(request, "LichessBroadcastTop").await
    }

    /// Searches broadcasts. `GET /api/broadcast/search`
    pub async fn search(&self, query: &str, page: u32) -> Result<LichessBroadcastSearchPage> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/broadcast/search")
            .query(&[("q", query), ("page", &page.to_string())]);
        http::json(request, "LichessBroadcastSearchPage").await
    }

    /// Streams broadcasts created by a user. `GET /api/broadcast/by/{username}`
    pub async fn by_user(
        &self,
        username: &str,
    ) -> Result<BoxStream<'static, Result<LichessBroadcast>>> {
        let path = format!("/api/broadcast/by/{}", http::segment(username));
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::stream(request, self.client.max_line_bytes()).await
    }

    /// Streams the authenticated user's broadcast rounds.
    /// `GET /api/broadcast/my-rounds`
    pub async fn my_rounds(&self) -> Result<BoxStream<'static, Result<LichessBroadcastMyRound>>> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/broadcast/my-rounds");
        http::stream(request, self.client.max_line_bytes()).await
    }

    /// Gets a broadcast tournament with its rounds.
    /// `GET /api/broadcast/{broadcastTournamentId}`
    pub async fn get_tournament(&self, tournament_id: &str) -> Result<LichessBroadcast> {
        let path = format!("/api/broadcast/{}", http::segment(tournament_id));
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "LichessBroadcast").await
    }

    /// Gets a round with its games.
    /// `GET /api/broadcast/{tourSlug}/{roundSlug}/{roundId}`
    pub async fn round(
        &self,
        tour_slug: &str,
        round_slug: &str,
        round_id: &str,
    ) -> Result<LichessBroadcastRoundView> {
        let path = format!(
            "/api/broadcast/{}/{}/{}",
            http::segment(tour_slug),
            http::segment(round_slug),
            http::segment(round_id)
        );
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "LichessBroadcastRoundView").await
    }

    /// Exports a round as PGN. `GET /api/broadcast/round/{roundId}.pgn`
    pub async fn round_pgn(&self, round_id: &str) -> Result<String> {
        let path = format!("/api/broadcast/round/{}.pgn", http::segment(round_id));
        http::text(self.client.request(Method::GET, Host::Default, &path)).await
    }

    /// Exports all rounds of a tournament as PGN.
    /// `GET /api/broadcast/{broadcastTournamentId}.pgn`
    ///
    /// For real-time updates about an ongoing tournament, prefer the round PGN
    /// stream ([`Self::stream_round_pgn`]) or group PGN stream
    /// ([`Self::stream_group_pgn`]) instead.
    pub async fn all_rounds_pgn(&self, tournament_id: &str) -> Result<String> {
        let path = format!("/api/broadcast/{}.pgn", http::segment(tournament_id));
        http::text(self.client.request(Method::GET, Host::Default, &path)).await
    }

    /// Streams a round's PGN as games are updated (text; stays open while the
    /// round is live). `GET /api/stream/broadcast/round/{roundId}.pgn`
    pub async fn stream_round_pgn(&self, round_id: &str) -> Result<String> {
        let path = format!(
            "/api/stream/broadcast/round/{}.pgn",
            http::segment(round_id)
        );
        http::text(self.client.request(Method::GET, Host::Default, &path)).await
    }

    /// Streams the PGN of all ongoing rounds of a broadcast group as games are
    /// updated (text; stays open while rounds are live).
    /// `GET /api/stream/broadcast/group/{broadcastGroupId}.pgn`
    pub async fn stream_group_pgn(&self, group_id: &str) -> Result<String> {
        let path = format!(
            "/api/stream/broadcast/group/{}.pgn",
            http::segment(group_id)
        );
        http::text(self.client.request(Method::GET, Host::Default, &path)).await
    }

    /// Pushes PGN games to a round.
    /// `POST /api/broadcast/round/{roundId}/push`
    pub async fn push_pgn(&self, round_id: &str, pgn: &str) -> Result<LichessBroadcastPushResult> {
        let path = format!("/api/broadcast/round/{}/push", http::segment(round_id));
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .header(CONTENT_TYPE, "text/plain")
            .body(pgn.to_owned());
        http::json(request, "LichessBroadcastPushResult").await
    }

    /// Resets a round, removing all its games.
    /// `POST /api/broadcast/round/{roundId}/reset`
    pub async fn reset_round(&self, round_id: &str) -> Result<()> {
        let path = format!("/api/broadcast/round/{}/reset", http::segment(round_id));
        http::ok(self.client.request(Method::POST, Host::Default, &path)).await
    }

    /// Lists the players of a broadcast. `GET /broadcast/{id}/players`
    pub async fn players(&self, tournament_id: &str) -> Result<Vec<LichessBroadcastPlayerEntry>> {
        let path = format!("/broadcast/{}/players", http::segment(tournament_id));
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "Vec<LichessBroadcastPlayerEntry>").await
    }

    /// Gets a single player of a broadcast.
    /// `GET /broadcast/{id}/players/{playerId}`
    pub async fn player(
        &self,
        tournament_id: &str,
        player_id: &str,
    ) -> Result<LichessBroadcastPlayerEntry> {
        let path = format!(
            "/broadcast/{}/players/{}",
            http::segment(tournament_id),
            http::segment(player_id)
        );
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "LichessBroadcastPlayerEntry").await
    }

    /// Gets the team leaderboard of a broadcast.
    /// `GET /broadcast/{id}/teams/standings`
    pub async fn team_standings(
        &self,
        tournament_id: &str,
    ) -> Result<Vec<LichessBroadcastPlayerEntry>> {
        let path = format!(
            "/broadcast/{}/teams/standings",
            http::segment(tournament_id)
        );
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "broadcast team standings").await
    }

    /// Starts building a new broadcast tournament. `POST /broadcast/new`
    #[must_use]
    pub fn create_tour(&self, name: &'a str) -> TourRequest<'a> {
        TourRequest::new(self.client, None, name)
    }

    /// Starts editing a broadcast tournament. `POST /broadcast/{id}/edit`
    #[must_use]
    pub fn update_tour(&self, tournament_id: &'a str, name: &'a str) -> TourRequest<'a> {
        TourRequest::new(self.client, Some(tournament_id), name)
    }

    /// Starts creating a round under a tournament. `POST /broadcast/{id}/new`
    #[must_use]
    pub fn create_round(&self, tournament_id: &'a str, name: &'a str) -> RoundRequest<'a> {
        RoundRequest::new(self.client, tournament_id, false, name)
    }

    /// Starts editing a round. `POST /broadcast/round/{roundId}/edit`
    #[must_use]
    pub fn update_round(&self, round_id: &'a str, name: &'a str) -> RoundRequest<'a> {
        RoundRequest::new(self.client, round_id, true, name)
    }
}

impl LichessClient {
    /// Broadcasts API: tournaments, rounds, players, and PGN.
    #[must_use]
    pub fn broadcasts(&self) -> BroadcastsApi<'_> {
        BroadcastsApi::new(self)
    }
}

/// Form body for creating/editing a broadcast tournament.
#[derive(Debug, Serialize)]
struct TourForm<'a> {
    name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    info: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    visibility: Option<&'a str>,
}

/// Builder for creating or editing a broadcast tournament.
#[derive(Debug)]
pub struct TourRequest<'a> {
    client: &'a LichessClient,
    edit_id: Option<&'a str>,
    form: TourForm<'a>,
}

impl<'a> TourRequest<'a> {
    /// Creates the request builder.
    pub(crate) fn new(client: &'a LichessClient, edit_id: Option<&'a str>, name: &'a str) -> Self {
        Self {
            client,
            edit_id,
            form: TourForm {
                name,
                info: None,
                visibility: None,
            },
        }
    }

    /// Sets the short description shown on the broadcast.
    #[must_use]
    pub fn info(mut self, info: &'a str) -> Self {
        self.form.info = Some(info);
        self
    }

    /// Sets the visibility (`public`, `unlisted`, or `private`).
    #[must_use]
    pub fn visibility(mut self, visibility: &'a str) -> Self {
        self.form.visibility = Some(visibility);
        self
    }

    /// Creates or updates the tournament.
    pub async fn send(self) -> Result<LichessBroadcast> {
        let path = match self.edit_id {
            Some(id) => format!("/broadcast/{}/edit", http::segment(id)),
            None => "/broadcast/new".to_owned(),
        };
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .form(&self.form);
        http::json(request, "LichessBroadcast").await
    }
}

/// Form body for creating/editing a broadcast round.
#[derive(Debug, Serialize)]
struct RoundForm<'a> {
    name: &'a str,
    #[serde(rename = "syncUrl", skip_serializing_if = "Option::is_none")]
    sync_url: Option<&'a str>,
    #[serde(rename = "startsAt", skip_serializing_if = "Option::is_none")]
    starts_at: Option<i64>,
}

/// Builder for creating a round (under a tournament) or editing a round.
#[derive(Debug)]
pub struct RoundRequest<'a> {
    client: &'a LichessClient,
    /// Tournament id when creating, or round id when editing.
    target_id: &'a str,
    edit: bool,
    form: RoundForm<'a>,
}

impl<'a> RoundRequest<'a> {
    /// Creates the request builder.
    pub(crate) fn new(
        client: &'a LichessClient,
        target_id: &'a str,
        edit: bool,
        name: &'a str,
    ) -> Self {
        Self {
            client,
            target_id,
            edit,
            form: RoundForm {
                name,
                sync_url: None,
                starts_at: None,
            },
        }
    }

    /// Sets a source URL to automatically sync games from.
    #[must_use]
    pub fn sync_url(mut self, url: &'a str) -> Self {
        self.form.sync_url = Some(url);
        self
    }

    /// Sets the round start time (Unix milliseconds).
    #[must_use]
    pub fn starts_at(mut self, timestamp: i64) -> Self {
        self.form.starts_at = Some(timestamp);
        self
    }

    /// Creates or updates the round.
    pub async fn send(self) -> Result<LichessBroadcastRoundView> {
        let path = if self.edit {
            format!("/broadcast/round/{}/edit", http::segment(self.target_id))
        } else {
            format!("/broadcast/{}/new", http::segment(self.target_id))
        };
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .form(&self.form);
        http::json(request, "LichessBroadcastRoundView").await
    }
}

/// A broadcast tournament's metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessBroadcastTour {
    /// The tournament id.
    pub id: String,
    /// The tournament name.
    pub name: String,
    /// The URL slug.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    /// The description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The canonical URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// The promotion tier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tier: Option<i32>,
    /// Creation time (Unix milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    /// The cover image URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
}

/// A round within a broadcast tournament.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessBroadcastRoundInfo {
    /// The round id.
    pub id: String,
    /// The round name.
    pub name: String,
    /// The URL slug.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    /// The canonical URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Creation time (Unix milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    /// Whether the round is rated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rated: Option<bool>,
    /// Whether the round is currently ongoing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ongoing: Option<bool>,
    /// Start time (Unix milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub starts_at: Option<i64>,
    /// Finish time (Unix milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<i64>,
    /// Whether the round has finished.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished: Option<bool>,
}

/// A broadcast tournament together with its rounds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessBroadcast {
    /// The tournament.
    pub tour: LichessBroadcastTour,
    /// The group this tournament belongs to, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// The rounds.
    #[serde(default)]
    pub rounds: Vec<LichessBroadcastRoundInfo>,
    /// The id of the round shown by default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_round_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_broadcast_with_rounds() {
        let json = r#"{"tour":{"id":"abc","name":"World Champ","slug":"wc"},
            "rounds":[{"id":"r1","name":"Round 1","slug":"round-1","url":"u",
                       "createdAt":1,"rated":true,"finished":false}]}"#;
        let broadcast: LichessBroadcast = serde_json::from_str(json).unwrap();
        assert_eq!(broadcast.tour.name, "World Champ");
        assert_eq!(broadcast.rounds[0].id, "r1");
        assert_eq!(broadcast.rounds[0].finished, Some(false));
    }
}

/// The top broadcasts. `GET /api/broadcast/top`
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessBroadcastTop {
    /// Currently-active broadcasts.
    #[serde(default)]
    pub active: Vec<LichessBroadcast>,
    /// Upcoming broadcasts.
    #[serde(default)]
    pub upcoming: Vec<LichessBroadcast>,
    /// A page of past broadcasts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub past: Option<Value>,
}

/// A paginated page of broadcast search results.
/// `GET /api/broadcast/search`
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessBroadcastSearchPage {
    /// The current page number.
    pub current_page: u32,
    /// Maximum results per page.
    pub max_per_page: u32,
    /// The broadcasts on this page.
    #[serde(default)]
    pub current_page_results: Vec<LichessBroadcast>,
    /// The previous page number, if any.
    #[serde(default)]
    pub previous_page: Option<u32>,
    /// The next page number, if any.
    #[serde(default)]
    pub next_page: Option<u32>,
}

/// A round with its games. `GET /api/broadcast/{tourSlug}/{roundSlug}/{roundId}`
///
/// The deeply-nested round/study/games payloads are preserved verbatim.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessBroadcastRoundView {
    /// The parent tournament.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tour: Option<LichessBroadcastTour>,
    /// The round info.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub round: Option<Value>,
    /// The games in the round.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub games: Option<Value>,
    /// The backing study info.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub study: Option<Value>,
}

/// A player entry in a broadcast. `GET /broadcast/{id}/players`
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessBroadcastPlayerEntry {
    /// The player's name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// All other fields, preserved verbatim.
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

/// One of the authenticated user's broadcast rounds.
/// `GET /api/broadcast/my-rounds`
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessBroadcastMyRound {
    /// All fields, preserved verbatim (round + tour + study).
    #[serde(flatten)]
    pub data: HashMap<String, Value>,
}

/// The result of pushing PGN to a round.
/// `POST /api/broadcast/round/{roundId}/push`
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessBroadcastPushResult {
    /// All fields, preserved verbatim.
    #[serde(flatten)]
    pub data: HashMap<String, Value>,
}
