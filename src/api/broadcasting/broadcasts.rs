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
use crate::model::PgnExportOptions;

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
    pub async fn round_pgn(&self, round_id: &str, options: &PgnExportOptions) -> Result<String> {
        let path = format!("/api/broadcast/round/{}.pgn", http::segment(round_id));
        let request = self
            .client
            .request(Method::GET, Host::Default, &path)
            .query(options);
        http::text(request).await
    }

    /// Exports all rounds of a tournament as PGN.
    /// `GET /api/broadcast/{broadcastTournamentId}.pgn`
    pub async fn all_rounds_pgn(
        &self,
        tournament_id: &str,
        options: &PgnExportOptions,
    ) -> Result<String> {
        let path = format!("/api/broadcast/{}.pgn", http::segment(tournament_id));
        let request = self
            .client
            .request(Method::GET, Host::Default, &path)
            .query(options);
        http::text(request).await
    }

    /// Streams a round's PGN as games are updated (text; stays open while the
    /// round is live). `GET /api/stream/broadcast/round/{roundId}.pgn`
    pub async fn stream_round_pgn(
        &self,
        round_id: &str,
        options: &PgnExportOptions,
    ) -> Result<String> {
        let path = format!(
            "/api/stream/broadcast/round/{}.pgn",
            http::segment(round_id)
        );
        let request = self
            .client
            .request(Method::GET, Host::Default, &path)
            .query(options);
        http::text(request).await
    }

    /// Streams the PGN of all ongoing rounds of a broadcast group as games are
    /// updated (text; stays open while rounds are live).
    /// `GET /api/stream/broadcast/group/{broadcastGroupId}.pgn`
    pub async fn stream_group_pgn(
        &self,
        group_id: &str,
        options: &PgnExportOptions,
    ) -> Result<String> {
        let path = format!(
            "/api/stream/broadcast/group/{}.pgn",
            http::segment(group_id)
        );
        let request = self
            .client
            .request(Method::GET, Host::Default, &path)
            .query(options);
        http::text(request).await
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
    ///
    /// The edit endpoint **replaces** the round: any field left unset is blanked
    /// (dropping the existing sync source, start time, etc.). Call
    /// [`RoundRequest::patch`] with `true` to instead update only the fields you
    /// set and leave the rest untouched.
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

/// Form body for creating/editing a broadcast tournament (flat, non-`info`
/// fields).
#[derive(Debug, Default, Serialize)]
struct TourForm<'a> {
    name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    visibility: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    markdown: Option<&'a str>,
    #[serde(rename = "showScores", skip_serializing_if = "Option::is_none")]
    show_scores: Option<bool>,
    #[serde(rename = "showRatingDiffs", skip_serializing_if = "Option::is_none")]
    show_rating_diffs: Option<bool>,
    #[serde(rename = "teamTable", skip_serializing_if = "Option::is_none")]
    team_table: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    players: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    teams: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tier: Option<u8>,
}

/// Display information for a broadcast tournament, serialized as `info.*` keys.
#[derive(Debug, Clone, Default, Serialize)]
pub struct BroadcastTourInfo<'a> {
    #[serde(rename = "info.format", skip_serializing_if = "Option::is_none")]
    format: Option<&'a str>,
    #[serde(rename = "info.tc", skip_serializing_if = "Option::is_none")]
    tc: Option<&'a str>,
    #[serde(rename = "info.fideTC", skip_serializing_if = "Option::is_none")]
    fide_tc: Option<&'a str>,
    #[serde(rename = "info.timeZone", skip_serializing_if = "Option::is_none")]
    time_zone: Option<&'a str>,
    #[serde(rename = "info.location", skip_serializing_if = "Option::is_none")]
    location: Option<&'a str>,
    #[serde(rename = "info.players", skip_serializing_if = "Option::is_none")]
    players: Option<&'a str>,
    #[serde(rename = "info.website", skip_serializing_if = "Option::is_none")]
    website: Option<&'a str>,
    #[serde(rename = "info.standings", skip_serializing_if = "Option::is_none")]
    standings: Option<&'a str>,
    #[serde(rename = "info.regulations", skip_serializing_if = "Option::is_none")]
    regulations: Option<&'a str>,
}

impl<'a> BroadcastTourInfo<'a> {
    /// Tournament format, e.g. `"8-player round-robin"`.
    #[must_use]
    pub fn format(mut self, format: &'a str) -> Self {
        self.format = Some(format);
        self
    }

    /// Time control, e.g. `"Classical"` or `"Rapid & Blitz"`.
    #[must_use]
    pub fn tc(mut self, tc: &'a str) -> Self {
        self.tc = Some(tc);
        self
    }

    /// FIDE rating category (`standard`, `rapid`, or `blitz`).
    #[must_use]
    pub fn fide_tc(mut self, fide_tc: &'a str) -> Self {
        self.fide_tc = Some(fide_tc);
        self
    }

    /// Timezone identifier, e.g. `America/New_York`.
    #[must_use]
    pub fn time_zone(mut self, time_zone: &'a str) -> Self {
        self.time_zone = Some(time_zone);
        self
    }

    /// Tournament location.
    #[must_use]
    pub fn location(mut self, location: &'a str) -> Self {
        self.location = Some(location);
        self
    }

    /// Up to four of the best participating players.
    #[must_use]
    pub fn players(mut self, players: &'a str) -> Self {
        self.players = Some(players);
        self
    }

    /// Official website URL.
    #[must_use]
    pub fn website(mut self, website: &'a str) -> Self {
        self.website = Some(website);
        self
    }

    /// Official standings website URL.
    #[must_use]
    pub fn standings(mut self, standings: &'a str) -> Self {
        self.standings = Some(standings);
        self
    }

    /// External URL to the tournament regulations.
    #[must_use]
    pub fn regulations(mut self, regulations: &'a str) -> Self {
        self.regulations = Some(regulations);
        self
    }
}

/// Builder for creating or editing a broadcast tournament.
#[derive(Debug)]
pub struct TourRequest<'a> {
    client: &'a LichessClient,
    edit_id: Option<&'a str>,
    form: TourForm<'a>,
    info: BroadcastTourInfo<'a>,
}

impl<'a> TourRequest<'a> {
    /// Creates the request builder.
    pub(crate) fn new(client: &'a LichessClient, edit_id: Option<&'a str>, name: &'a str) -> Self {
        Self {
            client,
            edit_id,
            form: TourForm {
                name,
                ..Default::default()
            },
            info: BroadcastTourInfo::default(),
        }
    }

    /// Sets the structured display information.
    #[must_use]
    pub fn info(mut self, info: BroadcastTourInfo<'a>) -> Self {
        self.info = info;
        self
    }

    /// Sets the visibility (`public`, `unlisted`, or `private`).
    #[must_use]
    pub fn visibility(mut self, visibility: &'a str) -> Self {
        self.form.visibility = Some(visibility);
        self
    }

    /// Sets a long Markdown description.
    #[must_use]
    pub fn markdown(mut self, markdown: &'a str) -> Self {
        self.form.markdown = Some(markdown);
        self
    }

    /// Sets whether to show player scores.
    #[must_use]
    pub fn show_scores(mut self, value: bool) -> Self {
        self.form.show_scores = Some(value);
        self
    }

    /// Sets whether to show rating differences.
    #[must_use]
    pub fn show_rating_diffs(mut self, value: bool) -> Self {
        self.form.show_rating_diffs = Some(value);
        self
    }

    /// Sets whether to display a team table.
    #[must_use]
    pub fn team_table(mut self, value: bool) -> Self {
        self.form.team_table = Some(value);
        self
    }

    /// Sets player tags / overrides (one line per player).
    #[must_use]
    pub fn players(mut self, players: &'a str) -> Self {
        self.form.players = Some(players);
        self
    }

    /// Assigns players to teams (one line per player).
    #[must_use]
    pub fn teams(mut self, teams: &'a str) -> Self {
        self.form.teams = Some(teams);
        self
    }

    /// Sets the broadcast tier (`3`, `4`, or `5`).
    #[must_use]
    pub fn tier(mut self, tier: u8) -> Self {
        self.form.tier = Some(tier);
        self
    }

    // NOTE: `tiebreaks` (an array) and `grouping` (a nested object with an
    // inner array) are intentionally not exposed yet — they need array /
    // nested form-key encoding that the flat value-type approach does not
    // cover. Tracked as a follow-up gap.

    /// Creates or updates the tournament.
    pub async fn send(self) -> Result<LichessBroadcast> {
        let path = match self.edit_id {
            Some(id) => format!("/broadcast/{}/edit", http::segment(id)),
            None => "/broadcast/new".to_owned(),
        };
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .form_parts(&self.form, &self.info);
        http::json(request, "LichessBroadcast").await
    }
}

/// Form body for creating/editing a broadcast round.
#[derive(Debug, Default, Serialize)]
struct RoundForm<'a> {
    name: &'a str,
    #[serde(rename = "syncUrl", skip_serializing_if = "Option::is_none")]
    sync_url: Option<&'a str>,
    #[serde(rename = "syncUrls", skip_serializing_if = "Option::is_none")]
    sync_urls: Option<&'a str>,
    #[serde(rename = "syncIds", skip_serializing_if = "Option::is_none")]
    sync_ids: Option<&'a str>,
    #[serde(rename = "syncUsers", skip_serializing_if = "Option::is_none")]
    sync_users: Option<&'a str>,
    #[serde(rename = "onlyRound", skip_serializing_if = "Option::is_none")]
    only_round: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    slices: Option<&'a str>,
    #[serde(rename = "syncSource", skip_serializing_if = "Option::is_none")]
    sync_source: Option<&'a str>,
    #[serde(rename = "startsAt", skip_serializing_if = "Option::is_none")]
    starts_at: Option<i64>,
    #[serde(
        rename = "startsAfterPrevious",
        skip_serializing_if = "Option::is_none"
    )]
    starts_after_previous: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    delay: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    period: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rated: Option<bool>,
    #[serde(rename = "customScoring", skip_serializing_if = "Option::is_none")]
    custom_scoring: Option<&'a str>,
    #[serde(rename = "teamCustomScoring", skip_serializing_if = "Option::is_none")]
    team_custom_scoring: Option<&'a str>,
}

/// Builder for creating a round (under a tournament) or editing a round.
#[derive(Debug)]
pub struct RoundRequest<'a> {
    client: &'a LichessClient,
    /// Tournament id when creating, or round id when editing.
    target_id: &'a str,
    edit: bool,
    patch: Option<bool>,
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
            patch: None,
            form: RoundForm {
                name,
                ..Default::default()
            },
        }
    }

    /// Sets a single source URL to automatically sync games from.
    #[must_use]
    pub fn sync_url(mut self, url: &'a str) -> Self {
        self.form.sync_url = Some(url);
        self
    }

    /// Sets multiple source URLs to sync games from (newline-separated).
    #[must_use]
    pub fn sync_urls(mut self, urls: &'a str) -> Self {
        self.form.sync_urls = Some(urls);
        self
    }

    /// Syncs games from these Lichess game ids (space/newline-separated).
    #[must_use]
    pub fn sync_ids(mut self, ids: &'a str) -> Self {
        self.form.sync_ids = Some(ids);
        self
    }

    /// Syncs games from these Lichess usernames.
    #[must_use]
    pub fn sync_users(mut self, users: &'a str) -> Self {
        self.form.sync_users = Some(users);
        self
    }

    /// Only import games matching this PGN `Round` tag.
    #[must_use]
    pub fn only_round(mut self, round: u32) -> Self {
        self.form.only_round = Some(round);
        self
    }

    /// Selects a subset of games from the source (slice expression).
    #[must_use]
    pub fn slices(mut self, slices: &'a str) -> Self {
        self.form.slices = Some(slices);
        self
    }

    /// Sets the sync source.
    #[must_use]
    pub fn sync_source(mut self, source: &'a str) -> Self {
        self.form.sync_source = Some(source);
        self
    }

    /// Sets the round start time (Unix milliseconds).
    #[must_use]
    pub fn starts_at(mut self, timestamp: i64) -> Self {
        self.form.starts_at = Some(timestamp);
        self
    }

    /// Starts the round automatically after the previous one finishes.
    #[must_use]
    pub fn starts_after_previous(mut self, value: bool) -> Self {
        self.form.starts_after_previous = Some(value);
        self
    }

    /// Sets the broadcast delay, in seconds.
    #[must_use]
    pub fn delay(mut self, seconds: u32) -> Self {
        self.form.delay = Some(seconds);
        self
    }

    /// Sets the source polling period, in seconds.
    #[must_use]
    pub fn period(mut self, seconds: u32) -> Self {
        self.form.period = Some(seconds);
        self
    }

    /// Sets the round status.
    #[must_use]
    pub fn status(mut self, status: &'a str) -> Self {
        self.form.status = Some(status);
        self
    }

    /// Sets whether the round's games are rated.
    #[must_use]
    pub fn rated(mut self, value: bool) -> Self {
        self.form.rated = Some(value);
        self
    }

    /// Sets a custom scoring configuration.
    #[must_use]
    pub fn custom_scoring(mut self, scoring: &'a str) -> Self {
        self.form.custom_scoring = Some(scoring);
        self
    }

    /// Sets a custom team scoring configuration.
    #[must_use]
    pub fn team_custom_scoring(mut self, scoring: &'a str) -> Self {
        self.form.team_custom_scoring = Some(scoring);
        self
    }

    /// On an edit, merges the given fields rather than replacing the round
    /// (`patch` query param).
    #[must_use]
    pub fn patch(mut self, value: bool) -> Self {
        self.patch = Some(value);
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
            .query(&[("patch", self.patch)])
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

    #[test]
    fn tour_info_serializes_to_dotted_keys() {
        let query = serde_urlencoded::to_string(
            BroadcastTourInfo::default()
                .format("8-player RR")
                .fide_tc("standard"),
        )
        .unwrap();
        assert!(query.contains("info.format=8-player+RR"));
        assert!(query.contains("info.fideTC=standard"));
    }

    #[test]
    fn empty_tour_info_serializes_to_nothing() {
        assert_eq!(
            serde_urlencoded::to_string(BroadcastTourInfo::default()).unwrap(),
            ""
        );
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
