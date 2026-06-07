//! The Teams API: discover teams and manage membership.
//!
//! Reached through [`LichessClient::teams`].

use futures_util::stream::BoxStream;
use reqwest::Method;
use serde::Serialize;

use crate::api::tournaments::arena::LichessArena;
use crate::api::tournaments::swiss::LichessSwiss;
use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;
use crate::model::LichessUser;

mod model;

pub use model::{
    LichessTeam, LichessTeamPaginator, LichessTeamRequest, LichessTeamRequestWithUser,
};

/// Form body for joining a team.
#[derive(Debug, Serialize)]
struct JoinForm<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    password: Option<&'a str>,
}

/// Accessor for the Teams API.
#[derive(Debug)]
pub struct TeamsApi<'a> {
    client: &'a LichessClient,
}

impl<'a> TeamsApi<'a> {
    /// Binds the accessor to a client.
    pub(crate) fn new(client: &'a LichessClient) -> Self {
        Self { client }
    }

    /// Gets a team by id. `GET /api/team/{teamId}`
    pub async fn get(&self, team_id: &str) -> Result<LichessTeam> {
        let path = format!("/api/team/{team_id}");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "LichessTeam").await
    }

    /// Lists the most popular teams, paginated. `GET /api/team/all`
    pub async fn all(&self, page: u32) -> Result<LichessTeamPaginator> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/team/all")
            .query(&[("page", page)]);
        http::json(request, "LichessTeamPaginator").await
    }

    /// Lists the teams a user belongs to. `GET /api/team/of/{username}`
    pub async fn of_user(&self, username: &str) -> Result<Vec<LichessTeam>> {
        let path = format!("/api/team/of/{username}");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "Vec<LichessTeam>").await
    }

    /// Searches teams by text, paginated. `GET /api/team/search`
    pub async fn search(&self, text: &str, page: u32) -> Result<LichessTeamPaginator> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/team/search")
            .query(&[("text", text), ("page", &page.to_string())]);
        http::json(request, "LichessTeamPaginator").await
    }

    /// Streams the members of a team. `GET /api/team/{teamId}/users`
    pub async fn members(&self, team_id: &str) -> Result<BoxStream<'static, Result<LichessUser>>> {
        let path = format!("/api/team/{team_id}/users");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::stream(request).await
    }

    /// Joins a team, optionally with a message and entry password.
    /// `POST /team/{teamId}/join`
    pub async fn join(
        &self,
        team_id: &str,
        message: Option<&str>,
        password: Option<&str>,
    ) -> Result<()> {
        let path = format!("/team/{team_id}/join");
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .form(&JoinForm { message, password });
        http::ok(request).await
    }

    /// Leaves a team. `POST /team/{teamId}/quit`
    pub async fn quit(&self, team_id: &str) -> Result<()> {
        let path = format!("/team/{team_id}/quit");
        http::ok(self.client.request(Method::POST, Host::Default, &path)).await
    }

    /// Kicks a member from a team. `POST /api/team/{teamId}/kick/{userId}`
    pub async fn kick(&self, team_id: &str, user_id: &str) -> Result<()> {
        let path = format!("/api/team/{team_id}/kick/{user_id}");
        http::ok(self.client.request(Method::POST, Host::Default, &path)).await
    }

    /// Sends a private message to all members. `POST /team/{teamId}/pm-all`
    pub async fn message_all(&self, team_id: &str, message: &str) -> Result<()> {
        let path = format!("/team/{team_id}/pm-all");
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .form(&[("message", message)]);
        http::ok(request).await
    }

    /// Lists pending join requests. `GET /api/team/{teamId}/requests`
    pub async fn join_requests(&self, team_id: &str) -> Result<Vec<LichessTeamRequestWithUser>> {
        let path = format!("/api/team/{team_id}/requests");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "Vec<LichessTeamRequestWithUser>").await
    }

    /// Accepts a join request.
    /// `POST /api/team/{teamId}/request/{userId}/accept`
    pub async fn accept_request(&self, team_id: &str, user_id: &str) -> Result<()> {
        let path = format!("/api/team/{team_id}/request/{user_id}/accept");
        http::ok(self.client.request(Method::POST, Host::Default, &path)).await
    }

    /// Declines a join request.
    /// `POST /api/team/{teamId}/request/{userId}/decline`
    pub async fn decline_request(&self, team_id: &str, user_id: &str) -> Result<()> {
        let path = format!("/api/team/{team_id}/request/{user_id}/decline");
        http::ok(self.client.request(Method::POST, Host::Default, &path)).await
    }

    /// Streams the arena tournaments of a team (NDJSON).
    /// `GET /api/team/{teamId}/arena`
    pub async fn arena_tournaments(
        &self,
        team_id: &str,
    ) -> Result<BoxStream<'static, Result<LichessArena>>> {
        let path = format!("/api/team/{team_id}/arena");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::stream(request).await
    }

    /// Streams the swiss tournaments of a team (NDJSON).
    /// `GET /api/team/{teamId}/swiss`
    pub async fn swiss_tournaments(
        &self,
        team_id: &str,
    ) -> Result<BoxStream<'static, Result<LichessSwiss>>> {
        let path = format!("/api/team/{team_id}/swiss");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::stream(request).await
    }
}

impl LichessClient {
    /// Teams API: discover teams and manage membership.
    #[must_use]
    pub fn teams(&self) -> TeamsApi<'_> {
        TeamsApi::new(self)
    }
}
