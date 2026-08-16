//! Round creation/edit builder, its form types, and custom scoring.

use reqwest::Method;
use serde::Serialize;

use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;

use super::LichessBroadcastRoundView;

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
}

/// Points awarded for a win and a draw (each `0.0`–`10.0`).
///
/// Used both for a single color/team and, via [`BroadcastCustomScoring`], per color.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BroadcastCustomPoints {
    /// Points awarded for a win.
    pub win: f64,
    /// Points awarded for a draw.
    pub draw: f64,
}

impl BroadcastCustomPoints {
    /// Appends the `{prefix}.win` / `{prefix}.draw` form pairs.
    fn append_pairs(self, prefix: &str, out: &mut Vec<(String, String)>) {
        out.push((format!("{prefix}.win"), self.win.to_string()));
        out.push((format!("{prefix}.draw"), self.draw.to_string()));
    }
}

/// Scoring overrides for both colors of a broadcast round.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BroadcastCustomScoring {
    /// Points awarded when White wins or draws.
    pub white: BroadcastCustomPoints,
    /// Points awarded when Black wins or draws.
    pub black: BroadcastCustomPoints,
}

impl BroadcastCustomScoring {
    /// Appends the `customScoring.{color}.{win,draw}` form pairs.
    fn append_pairs(self, out: &mut Vec<(String, String)>) {
        self.white.append_pairs("customScoring.white", out);
        self.black.append_pairs("customScoring.black", out);
    }
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
    custom_scoring: Option<BroadcastCustomScoring>,
    team_custom_scoring: Option<BroadcastCustomPoints>,
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
            custom_scoring: None,
            team_custom_scoring: None,
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

    /// Overrides the points awarded for wins and draws, per color.
    #[must_use]
    pub fn custom_scoring(mut self, scoring: BroadcastCustomScoring) -> Self {
        self.custom_scoring = Some(scoring);
        self
    }

    /// Overrides the points awarded for a team-match win or draw.
    #[must_use]
    pub fn team_custom_scoring(mut self, scoring: BroadcastCustomPoints) -> Self {
        self.team_custom_scoring = Some(scoring);
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
        let core = serde_urlencoded::to_string(&self.form).unwrap_or_default();
        let scoring = serde_urlencoded::to_string(self.scoring_pairs()).unwrap_or_default();
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .query(&[("patch", self.patch)])
            .form_body(http::join_form(&[core, scoring]));
        http::json(request, "LichessBroadcastRoundView").await
    }

    /// Builds the nested `customScoring.*` / `teamCustomScoring.*` form pairs,
    /// which `serde_urlencoded` cannot express as struct fields.
    fn scoring_pairs(&self) -> Vec<(String, String)> {
        let mut pairs = Vec::new();
        if let Some(scoring) = self.custom_scoring {
            scoring.append_pairs(&mut pairs);
        }
        if let Some(team) = self.team_custom_scoring {
            team.append_pairs("teamCustomScoring", &mut pairs);
        }
        pairs
    }
}
