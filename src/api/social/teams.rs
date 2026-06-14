//! The Teams API: discover teams and manage membership.
//!
//! Reached through [`LichessClient::teams`].

use std::fmt;

use futures_util::stream::BoxStream;
use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::api::tournaments::arena::LichessArena;
use crate::api::tournaments::swiss::LichessSwiss;
use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;
use crate::model::{LichessLightUser, LichessUser};

/// Form body for joining a team.
///
/// The [`Debug`] output redacts the entry `password`.
#[derive(Serialize)]
struct JoinForm<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    password: Option<&'a str>,
}

impl fmt::Debug for JoinForm<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JoinForm")
            .field("message", &self.message)
            .field("password", &self.password.map(|_| "<redacted>"))
            .finish()
    }
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
        let path = format!("/api/team/{}", http::segment(team_id));
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
        let path = format!("/api/team/of/{}", http::segment(username));
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
        let path = format!("/api/team/{}/users", http::segment(team_id));
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
        let path = format!("/team/{}/join", http::segment(team_id));
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .form(&JoinForm { message, password });
        http::ok(request).await
    }

    /// Leaves a team. `POST /team/{teamId}/quit`
    pub async fn quit(&self, team_id: &str) -> Result<()> {
        let path = format!("/team/{}/quit", http::segment(team_id));
        http::ok(self.client.request(Method::POST, Host::Default, &path)).await
    }

    /// Kicks a member from a team. `POST /api/team/{teamId}/kick/{userId}`
    pub async fn kick(&self, team_id: &str, user_id: &str) -> Result<()> {
        let path = format!(
            "/api/team/{}/kick/{}",
            http::segment(team_id),
            http::segment(user_id)
        );
        http::ok(self.client.request(Method::POST, Host::Default, &path)).await
    }

    /// Sends a private message to all members. `POST /team/{teamId}/pm-all`
    pub async fn message_all(&self, team_id: &str, message: &str) -> Result<()> {
        let path = format!("/team/{}/pm-all", http::segment(team_id));
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .form(&[("message", message)]);
        http::ok(request).await
    }

    /// Lists pending join requests. `GET /api/team/{teamId}/requests`
    pub async fn join_requests(&self, team_id: &str) -> Result<Vec<LichessTeamRequestWithUser>> {
        let path = format!("/api/team/{}/requests", http::segment(team_id));
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "Vec<LichessTeamRequestWithUser>").await
    }

    /// Accepts a join request.
    /// `POST /api/team/{teamId}/request/{userId}/accept`
    pub async fn accept_request(&self, team_id: &str, user_id: &str) -> Result<()> {
        let path = format!(
            "/api/team/{}/request/{}/accept",
            http::segment(team_id),
            http::segment(user_id)
        );
        http::ok(self.client.request(Method::POST, Host::Default, &path)).await
    }

    /// Declines a join request.
    /// `POST /api/team/{teamId}/request/{userId}/decline`
    pub async fn decline_request(&self, team_id: &str, user_id: &str) -> Result<()> {
        let path = format!(
            "/api/team/{}/request/{}/decline",
            http::segment(team_id),
            http::segment(user_id)
        );
        http::ok(self.client.request(Method::POST, Host::Default, &path)).await
    }

    /// Streams the arena tournaments of a team (NDJSON).
    /// `GET /api/team/{teamId}/arena`
    pub async fn arena_tournaments(
        &self,
        team_id: &str,
    ) -> Result<BoxStream<'static, Result<LichessArena>>> {
        let path = format!("/api/team/{}/arena", http::segment(team_id));
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::stream(request).await
    }

    /// Streams the swiss tournaments of a team (NDJSON).
    /// `GET /api/team/{teamId}/swiss`
    pub async fn swiss_tournaments(
        &self,
        team_id: &str,
    ) -> Result<BoxStream<'static, Result<LichessSwiss>>> {
        let path = format!("/api/team/{}/swiss", http::segment(team_id));
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

/// A team.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessTeam {
    /// The team id.
    pub id: String,
    /// The team name.
    pub name: String,
    /// The team description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The team flair.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flair: Option<String>,
    /// The primary leader.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leader: Option<LichessLightUser>,
    /// All team leaders.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leaders: Option<Vec<LichessLightUser>>,
    /// The number of members.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nb_members: Option<u32>,
    /// Whether the team is open to join.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open: Option<bool>,
    /// Whether the authenticated user has joined.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub joined: Option<bool>,
    /// Whether the authenticated user has a pending join request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested: Option<bool>,
}

/// A paginated list of teams.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessTeamPaginator {
    /// The current page number.
    pub current_page: u32,
    /// The maximum results per page.
    pub max_per_page: u32,
    /// The teams on this page.
    pub current_page_results: Vec<LichessTeam>,
    /// The previous page number, if any.
    #[serde(default)]
    pub previous_page: Option<u32>,
    /// The next page number, if any.
    #[serde(default)]
    pub next_page: Option<u32>,
    /// The total number of results.
    pub nb_results: u32,
    /// The total number of pages.
    pub nb_pages: u32,
}

/// A pending join request on a team.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessTeamRequest {
    /// The team id.
    pub team_id: String,
    /// The requesting user id.
    pub user_id: String,
    /// When the request was made (Unix milliseconds).
    pub date: i64,
    /// The request message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// A join request together with the requesting user's profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessTeamRequestWithUser {
    /// The request.
    pub request: LichessTeamRequest,
    /// The requesting user.
    pub user: LichessUser,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_form_debug_redacts_password() {
        let form = JoinForm {
            message: Some("hi"),
            password: Some("supersecret"),
        };
        let debug = format!("{form:?}");
        assert!(!debug.contains("supersecret"));
        assert!(debug.contains("<redacted>"));
        assert!(debug.contains("hi"));
    }

    #[test]
    fn parses_team() {
        let json = r#"{"id":"coders","name":"Coders","nbMembers":42,"open":true,
            "leader":{"id":"t","name":"T"}}"#;
        let team: LichessTeam = serde_json::from_str(json).unwrap();
        assert_eq!(team.id, "coders");
        assert_eq!(team.nb_members, Some(42));
        assert_eq!(team.leader.unwrap().name, "T");
    }

    #[test]
    fn parses_paginator_with_null_pages() {
        let json = r#"{"currentPage":1,"maxPerPage":15,"currentPageResults":[],
            "previousPage":null,"nextPage":2,"nbResults":30,"nbPages":2}"#;
        let page: LichessTeamPaginator = serde_json::from_str(json).unwrap();
        assert_eq!(page.previous_page, None);
        assert_eq!(page.next_page, Some(2));
    }

    #[test]
    fn parses_request_with_user() {
        let json = r#"{"request":{"userId":"mary","teamId":"t","date":1,"message":"hi"},
            "user":{"id":"mary","username":"Mary"}}"#;
        let req: LichessTeamRequestWithUser = serde_json::from_str(json).unwrap();
        assert_eq!(req.request.user_id, "mary");
        assert_eq!(req.user.username, "Mary");
    }
}
