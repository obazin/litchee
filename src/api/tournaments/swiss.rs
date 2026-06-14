//! The Swiss Tournaments API: create, run, join, and export swiss tournaments.
//!
//! Reached through [`LichessClient::swiss`].

use futures_util::stream::BoxStream;
use reqwest::{Method, StatusCode};
use serde::{Deserialize, Serialize};

use crate::api::gameplay::games::LichessGame;
use crate::client::LichessClient;
use crate::config::Host;
use crate::error::{ApiErrorKind, LichessError, Result};
use crate::http;
use crate::model::LichessTitle;

/// Reclassifies a `401` from a Swiss edit/schedule request as the distinct
/// [`ApiErrorKind::SwissUnauthorizedEdit`] ownership rejection — the spec's
/// `SwissUnauthorisedEdit` response — rather than a generic auth failure.
fn map_unauthorized_edit(err: LichessError) -> LichessError {
    match err {
        LichessError::Api(api) if api.status == StatusCode::UNAUTHORIZED => {
            LichessError::Api(api.with_kind(ApiErrorKind::SwissUnauthorizedEdit))
        }
        other => other,
    }
}

/// Form body for creating a swiss tournament.
#[derive(Debug, Serialize)]
struct CreateForm<'a> {
    #[serde(rename = "clock.limit")]
    clock_limit: u32,
    #[serde(rename = "clock.increment")]
    clock_increment: u32,
    #[serde(rename = "nbRounds")]
    nb_rounds: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rated: Option<bool>,
    #[serde(rename = "roundInterval", skip_serializing_if = "Option::is_none")]
    round_interval: Option<u32>,
}

/// Accessor for the Swiss Tournaments API.
#[derive(Debug)]
pub struct SwissApi<'a> {
    client: &'a LichessClient,
}

impl<'a> SwissApi<'a> {
    /// Binds the accessor to a client.
    pub(crate) fn new(client: &'a LichessClient) -> Self {
        Self { client }
    }

    /// Gets a swiss tournament. `GET /api/swiss/{id}`
    pub async fn get(&self, id: &str) -> Result<LichessSwiss> {
        let path = format!("/api/swiss/{}", http::segment(id));
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "LichessSwiss").await
    }

    /// Creates a swiss tournament for a team. `POST /api/swiss/new/{teamId}`
    #[must_use]
    pub fn create(
        &self,
        team_id: &'a str,
        clock_limit: u32,
        clock_increment: u32,
        nb_rounds: u32,
    ) -> CreateSwissRequest<'a> {
        CreateSwissRequest::new(
            self.client,
            team_id,
            false,
            clock_limit,
            clock_increment,
            nb_rounds,
        )
    }

    /// Updates a swiss tournament. `POST /api/swiss/{id}/edit`
    #[must_use]
    pub fn edit(
        &self,
        id: &'a str,
        clock_limit: u32,
        clock_increment: u32,
        nb_rounds: u32,
    ) -> CreateSwissRequest<'a> {
        CreateSwissRequest::new(
            self.client,
            id,
            true,
            clock_limit,
            clock_increment,
            nb_rounds,
        )
    }

    /// Joins a swiss tournament. `POST /api/swiss/{id}/join`
    pub async fn join(&self, id: &str) -> Result<()> {
        self.post_action(id, "join").await
    }

    /// Withdraws from a swiss tournament. `POST /api/swiss/{id}/withdraw`
    pub async fn withdraw(&self, id: &str) -> Result<()> {
        self.post_action(id, "withdraw").await
    }

    /// Terminates a swiss tournament. `POST /api/swiss/{id}/terminate`
    pub async fn terminate(&self, id: &str) -> Result<()> {
        self.post_action(id, "terminate").await
    }

    /// Manually schedules the next round.
    /// `POST /api/swiss/{id}/schedule-next-round`
    pub async fn schedule_next_round(&self, id: &str) -> Result<()> {
        self.post_action(id, "schedule-next-round")
            .await
            .map_err(map_unauthorized_edit)
    }

    /// Downloads the tournament in TRF format. `GET /swiss/{id}.trf`
    pub async fn trf(&self, id: &str) -> Result<String> {
        let path = format!("/swiss/{}.trf", http::segment(id));
        http::text(self.client.request(Method::GET, Host::Default, &path)).await
    }

    /// Streams a swiss tournament's results. `GET /api/swiss/{id}/results`
    pub async fn results(
        &self,
        id: &str,
    ) -> Result<BoxStream<'static, Result<LichessSwissResult>>> {
        let path = format!("/api/swiss/{}/results", http::segment(id));
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::stream(request).await
    }

    /// Streams a swiss tournament's games as NDJSON. `GET /api/swiss/{id}/games`
    pub async fn games(&self, id: &str) -> Result<BoxStream<'static, Result<LichessGame>>> {
        let path = format!("/api/swiss/{}/games", http::segment(id));
        let request = self
            .client
            .request(Method::GET, Host::Default, &path)
            .header(reqwest::header::ACCEPT, "application/x-ndjson");
        http::stream(request).await
    }

    /// Issues a no-argument `POST` action on a swiss tournament.
    async fn post_action(&self, id: &str, action: &str) -> Result<()> {
        let path = format!("/api/swiss/{}/{}", http::segment(id), http::segment(action));
        http::ok(self.client.request(Method::POST, Host::Default, &path)).await
    }
}

/// Builder for creating or editing a swiss tournament.
#[derive(Debug)]
pub struct CreateSwissRequest<'a> {
    client: &'a LichessClient,
    /// Team id for creation, or tournament id when editing.
    target_id: &'a str,
    edit: bool,
    form: CreateForm<'a>,
}

impl<'a> CreateSwissRequest<'a> {
    /// Creates the request builder.
    fn new(
        client: &'a LichessClient,
        target_id: &'a str,
        edit: bool,
        clock_limit: u32,
        clock_increment: u32,
        nb_rounds: u32,
    ) -> Self {
        Self {
            client,
            target_id,
            edit,
            form: CreateForm {
                clock_limit,
                clock_increment,
                nb_rounds,
                name: None,
                rated: None,
                round_interval: None,
            },
        }
    }

    /// Sets the tournament name.
    #[must_use]
    pub fn name(mut self, name: &'a str) -> Self {
        self.form.name = Some(name);
        self
    }

    /// Sets whether the tournament is rated.
    #[must_use]
    pub fn rated(mut self, rated: bool) -> Self {
        self.form.rated = Some(rated);
        self
    }

    /// Sets the interval between rounds, in seconds.
    #[must_use]
    pub fn round_interval(mut self, seconds: u32) -> Self {
        self.form.round_interval = Some(seconds);
        self
    }

    /// Creates or updates the tournament.
    pub async fn send(self) -> Result<LichessSwiss> {
        let edit = self.edit;
        let path = if edit {
            format!("/api/swiss/{}/edit", http::segment(self.target_id))
        } else {
            format!("/api/swiss/new/{}", http::segment(self.target_id))
        };
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .form(&self.form);
        let result = http::json(request, "LichessSwiss").await;
        if edit {
            result.map_err(map_unauthorized_edit)
        } else {
            result
        }
    }
}

impl LichessClient {
    /// Swiss Tournaments API.
    #[must_use]
    pub fn swiss(&self) -> SwissApi<'_> {
        SwissApi::new(self)
    }
}

/// A swiss clock (`limit` + `increment`, in seconds).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessSwissClock {
    /// Initial time in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Increment per move in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub increment: Option<u32>,
}

/// When the next round of a swiss starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessSwissNextRound {
    /// Absolute start time (Unix milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub at: Option<i64>,
    /// Seconds until the next round starts.
    #[serde(rename = "in", default, skip_serializing_if = "Option::is_none")]
    pub in_seconds: Option<i64>,
}

/// A swiss tournament.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessSwiss {
    /// The tournament id.
    pub id: String,
    /// The creator's username.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    /// Start time (Unix milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub starts_at: Option<i64>,
    /// The tournament name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The clock.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clock: Option<LichessSwissClock>,
    /// The variant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    /// The current round number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub round: Option<u32>,
    /// The total number of rounds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nb_rounds: Option<u32>,
    /// The number of players.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nb_players: Option<u32>,
    /// The number of ongoing games.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nb_ongoing: Option<u32>,
    /// The status (`created`, `started`, or `finished`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Whether the tournament is rated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rated: Option<bool>,
    /// When the next round starts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_round: Option<LichessSwissNextRound>,
}

/// One entry in a swiss results stream.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessSwissResult {
    /// The player's final rank.
    pub rank: u32,
    /// The player's points.
    pub points: f64,
    /// The player's tie-break score.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tie_break: Option<f64>,
    /// The player's rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rating: Option<u32>,
    /// The player's username.
    pub username: String,
    /// The player's title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<LichessTitle>,
    /// The player's tournament performance rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub performance: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ApiError;

    #[test]
    fn maps_401_to_swiss_unauthorized_edit() {
        let err = LichessError::Api(ApiError::new(StatusCode::UNAUTHORIZED, None, None));
        match map_unauthorized_edit(err) {
            LichessError::Api(api) => assert_eq!(api.kind, ApiErrorKind::SwissUnauthorizedEdit),
            other => panic!("expected Api error, got {other:?}"),
        }
    }

    #[test]
    fn leaves_non_401_errors_unchanged() {
        let err = LichessError::Api(ApiError::new(StatusCode::NOT_FOUND, None, None));
        match map_unauthorized_edit(err) {
            LichessError::Api(api) => assert_eq!(api.kind, ApiErrorKind::NotFound),
            other => panic!("expected Api error, got {other:?}"),
        }
    }

    #[test]
    fn parses_swiss_with_next_round() {
        let json = r#"{"id":"abc","name":"Weekly","clock":{"limit":300,"increment":0},
            "variant":"standard","round":2,"nbRounds":7,"nbPlayers":40,
            "status":"started","rated":true,"nextRound":{"at":1700000000000,"in":120}}"#;
        let swiss: LichessSwiss = serde_json::from_str(json).unwrap();
        assert_eq!(swiss.nb_rounds, Some(7));
        assert_eq!(swiss.next_round.unwrap().in_seconds, Some(120));
    }

    #[test]
    fn parses_swiss_result_with_fractional_points() {
        let json = r#"{"rank":1,"points":5.5,"tieBreak":18.0,"username":"A","rating":2400}"#;
        let result: LichessSwissResult = serde_json::from_str(json).unwrap();
        assert!((result.points - 5.5).abs() < f64::EPSILON);
    }
}
