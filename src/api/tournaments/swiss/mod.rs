//! The Swiss Tournaments API: create, run, join, and export swiss tournaments.
//!
//! Reached through [`LichessClient::swiss`].

use futures_util::stream::BoxStream;
use reqwest::Method;
use serde::Serialize;

use crate::api::gameplay::games::LichessGame;
use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;

mod model;

pub use model::{LichessSwiss, LichessSwissClock, LichessSwissNextRound, LichessSwissResult};

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
        let path = format!("/api/swiss/{id}");
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
        self.post_action(id, "schedule-next-round").await
    }

    /// Downloads the tournament in TRF format. `GET /swiss/{id}.trf`
    pub async fn trf(&self, id: &str) -> Result<String> {
        let path = format!("/swiss/{id}.trf");
        http::text(self.client.request(Method::GET, Host::Default, &path)).await
    }

    /// Streams a swiss tournament's results. `GET /api/swiss/{id}/results`
    pub async fn results(
        &self,
        id: &str,
    ) -> Result<BoxStream<'static, Result<LichessSwissResult>>> {
        let path = format!("/api/swiss/{id}/results");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::stream(request).await
    }

    /// Streams a swiss tournament's games as NDJSON. `GET /api/swiss/{id}/games`
    pub async fn games(&self, id: &str) -> Result<BoxStream<'static, Result<LichessGame>>> {
        let path = format!("/api/swiss/{id}/games");
        let request = self
            .client
            .request(Method::GET, Host::Default, &path)
            .header(reqwest::header::ACCEPT, "application/x-ndjson");
        http::stream(request).await
    }

    /// Issues a no-argument `POST` action on a swiss tournament.
    async fn post_action(&self, id: &str, action: &str) -> Result<()> {
        let path = format!("/api/swiss/{id}/{action}");
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
        let path = if self.edit {
            format!("/api/swiss/{}/edit", self.target_id)
        } else {
            format!("/api/swiss/new/{}", self.target_id)
        };
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .form(&self.form);
        http::json(request, "LichessSwiss").await
    }
}

impl LichessClient {
    /// Swiss Tournaments API.
    #[must_use]
    pub fn swiss(&self) -> SwissApi<'_> {
        SwissApi::new(self)
    }
}
