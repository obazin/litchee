//! The Arena Tournaments API: create, run, join, and export arena tournaments.
//!
//! Reached through [`LichessClient::arena`].

use futures_util::stream::BoxStream;
use reqwest::Method;
use serde::Serialize;

use crate::api::gameplay::games::LichessGame;
use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;
use crate::model::LichessVariantKey;

mod model;

pub use model::{
    LichessArena, LichessArenaClock, LichessArenaFull, LichessArenaList, LichessArenaPerf,
    LichessArenaPlayer, LichessArenaResult, LichessArenaStanding, LichessTeamBattleStandings,
    LichessTeamBattleTeam,
};

/// Form body for creating an arena tournament.
#[derive(Debug, Serialize)]
struct CreateForm<'a> {
    name: &'a str,
    #[serde(rename = "clockTime")]
    clock_time: f32,
    #[serde(rename = "clockIncrement")]
    clock_increment: u32,
    minutes: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    rated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    variant: Option<LichessVariantKey>,
    #[serde(rename = "waitMinutes", skip_serializing_if = "Option::is_none")]
    wait_minutes: Option<u32>,
}

/// Accessor for the Arena Tournaments API.
#[derive(Debug)]
pub struct ArenaApi<'a> {
    client: &'a LichessClient,
}

impl<'a> ArenaApi<'a> {
    /// Binds the accessor to a client.
    pub(crate) fn new(client: &'a LichessClient) -> Self {
        Self { client }
    }

    /// Lists current arena tournaments. `GET /api/tournament`
    pub async fn list(&self) -> Result<LichessArenaList> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/tournament");
        http::json(request, "LichessArenaList").await
    }

    /// Gets full details of a tournament. `GET /api/tournament/{id}`
    pub async fn get(&self, id: &str) -> Result<LichessArenaFull> {
        let path = format!("/api/tournament/{id}");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "LichessArenaFull").await
    }

    /// Creates an arena tournament. `POST /api/tournament`
    #[must_use]
    pub fn create(
        &self,
        name: &'a str,
        clock_time: f32,
        clock_increment: u32,
        minutes: u32,
    ) -> CreateArenaRequest<'a> {
        CreateArenaRequest::new(
            self.client,
            None,
            name,
            clock_time,
            clock_increment,
            minutes,
        )
    }

    /// Updates an existing arena tournament. `POST /api/tournament/{id}`
    #[must_use]
    pub fn update(
        &self,
        id: &'a str,
        name: &'a str,
        clock_time: f32,
        clock_increment: u32,
        minutes: u32,
    ) -> CreateArenaRequest<'a> {
        CreateArenaRequest::new(
            self.client,
            Some(id),
            name,
            clock_time,
            clock_increment,
            minutes,
        )
    }

    /// Configures a team battle. `POST /api/tournament/team-battle/{id}`
    pub async fn setup_team_battle(
        &self,
        id: &str,
        team_ids: &[&str],
        nb_leaders: u32,
    ) -> Result<LichessArenaFull> {
        let path = format!("/api/tournament/team-battle/{id}");
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .form(&[
                ("teams", team_ids.join(",").as_str()),
                ("nbLeaders", nb_leaders.to_string().as_str()),
            ]);
        http::json(request, "LichessArenaFull").await
    }

    /// Gets the team standings of a team-battle arena.
    /// `GET /api/tournament/{id}/teams`
    pub async fn teams(&self, id: &str) -> Result<LichessTeamBattleStandings> {
        let path = format!("/api/tournament/{id}/teams");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "LichessTeamBattleStandings").await
    }

    /// Streams the arenas created by a user (NDJSON).
    /// `GET /api/user/{username}/tournament/created`
    pub async fn created_by(
        &self,
        username: &str,
    ) -> Result<BoxStream<'static, Result<LichessArena>>> {
        let path = format!("/api/user/{username}/tournament/created");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::stream(request).await
    }

    /// Streams the arenas a user has played (NDJSON).
    /// `GET /api/user/{username}/tournament/played`
    pub async fn played_by(
        &self,
        username: &str,
    ) -> Result<BoxStream<'static, Result<LichessArena>>> {
        let path = format!("/api/user/{username}/tournament/played");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::stream(request).await
    }

    /// Joins a tournament. `POST /api/tournament/{id}/join`
    pub async fn join(&self, id: &str) -> Result<()> {
        let path = format!("/api/tournament/{id}/join");
        http::ok(self.client.request(Method::POST, Host::Default, &path)).await
    }

    /// Withdraws from a tournament. `POST /api/tournament/{id}/withdraw`
    pub async fn withdraw(&self, id: &str) -> Result<()> {
        let path = format!("/api/tournament/{id}/withdraw");
        http::ok(self.client.request(Method::POST, Host::Default, &path)).await
    }

    /// Terminates a tournament you created. `POST /api/tournament/{id}/terminate`
    pub async fn terminate(&self, id: &str) -> Result<()> {
        let path = format!("/api/tournament/{id}/terminate");
        http::ok(self.client.request(Method::POST, Host::Default, &path)).await
    }

    /// Streams a tournament's results. `GET /api/tournament/{id}/results`
    pub async fn results(
        &self,
        id: &str,
    ) -> Result<BoxStream<'static, Result<LichessArenaResult>>> {
        let path = format!("/api/tournament/{id}/results");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::stream(request).await
    }

    /// Streams a tournament's games as NDJSON. `GET /api/tournament/{id}/games`
    pub async fn games(&self, id: &str) -> Result<BoxStream<'static, Result<LichessGame>>> {
        let path = format!("/api/tournament/{id}/games");
        let request = self
            .client
            .request(Method::GET, Host::Default, &path)
            .header(reqwest::header::ACCEPT, "application/x-ndjson");
        http::stream(request).await
    }
}

/// Builder for creating or updating an arena tournament.
#[derive(Debug)]
pub struct CreateArenaRequest<'a> {
    client: &'a LichessClient,
    id: Option<&'a str>,
    form: CreateForm<'a>,
}

impl<'a> CreateArenaRequest<'a> {
    /// Creates the request builder.
    fn new(
        client: &'a LichessClient,
        id: Option<&'a str>,
        name: &'a str,
        clock_time: f32,
        clock_increment: u32,
        minutes: u32,
    ) -> Self {
        Self {
            client,
            id,
            form: CreateForm {
                name,
                clock_time,
                clock_increment,
                minutes,
                rated: None,
                variant: None,
                wait_minutes: None,
            },
        }
    }

    /// Sets whether the tournament is rated.
    #[must_use]
    pub fn rated(mut self, rated: bool) -> Self {
        self.form.rated = Some(rated);
        self
    }

    /// Sets the variant.
    #[must_use]
    pub fn variant(mut self, variant: LichessVariantKey) -> Self {
        self.form.variant = Some(variant);
        self
    }

    /// Sets how many minutes to wait before starting.
    #[must_use]
    pub fn wait_minutes(mut self, minutes: u32) -> Self {
        self.form.wait_minutes = Some(minutes);
        self
    }

    /// Creates or updates the tournament.
    pub async fn send(self) -> Result<LichessArenaFull> {
        let path = match self.id {
            Some(id) => format!("/api/tournament/{id}"),
            None => "/api/tournament".to_owned(),
        };
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .form(&self.form);
        http::json(request, "LichessArenaFull").await
    }
}

impl LichessClient {
    /// Arena Tournaments API.
    #[must_use]
    pub fn arena(&self) -> ArenaApi<'_> {
        ArenaApi::new(self)
    }
}
