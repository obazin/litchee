//! The Challenges API: create, accept, decline, and manage challenges.
//!
//! Reached through [`LichessClient::challenges`].

use reqwest::Method;

use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;

use std::collections::HashMap;

mod create;
mod model;

pub use create::{AiChallengeRequest, ChallengeRequest, OpenChallengeRequest};
pub use model::{
    LichessChallenge, LichessChallengeColor, LichessChallengePerf, LichessChallengeStatus,
    LichessChallengeUser, LichessChallengeVariant, LichessChallenges, LichessOpenChallenge,
    LichessTimeControl,
};

/// Accessor for the Challenges API.
#[derive(Debug)]
pub struct ChallengesApi<'a> {
    client: &'a LichessClient,
}

impl<'a> ChallengesApi<'a> {
    /// Binds the accessor to a client.
    pub(crate) fn new(client: &'a LichessClient) -> Self {
        Self { client }
    }

    /// Lists the authenticated user's incoming and outgoing challenges.
    ///
    /// `GET /api/challenge`
    pub async fn list(&self) -> Result<LichessChallenges> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/challenge");
        http::json(request, "LichessChallenges").await
    }

    /// Shows a single challenge by id. `GET /api/challenge/{challengeId}/show`
    pub async fn show(&self, challenge_id: &str) -> Result<LichessChallenge> {
        let path = format!("/api/challenge/{challenge_id}/show");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "LichessChallenge").await
    }

    /// Starts building a challenge to a user. `POST /api/challenge/{username}`
    #[must_use]
    pub fn challenge(&self, username: &'a str) -> ChallengeRequest<'a> {
        ChallengeRequest::new(self.client, username)
    }

    /// Starts building a challenge against the AI. `POST /api/challenge/ai`
    #[must_use]
    pub fn challenge_ai(&self, level: u8) -> AiChallengeRequest<'a> {
        AiChallengeRequest::new(self.client, level)
    }

    /// Accepts an incoming challenge.
    /// `POST /api/challenge/{challengeId}/accept`
    pub async fn accept(&self, challenge_id: &str) -> Result<()> {
        let path = format!("/api/challenge/{challenge_id}/accept");
        http::ok(self.client.request(Method::POST, Host::Default, &path)).await
    }

    /// Declines an incoming challenge, optionally with a reason key.
    /// `POST /api/challenge/{challengeId}/decline`
    pub async fn decline(&self, challenge_id: &str, reason: Option<&str>) -> Result<()> {
        let path = format!("/api/challenge/{challenge_id}/decline");
        let mut request = self.client.request(Method::POST, Host::Default, &path);
        if let Some(reason) = reason {
            request = request.form(&[("reason", reason)]);
        }
        http::ok(request).await
    }

    /// Cancels a challenge you sent.
    /// `POST /api/challenge/{challengeId}/cancel`
    pub async fn cancel(&self, challenge_id: &str) -> Result<()> {
        let path = format!("/api/challenge/{challenge_id}/cancel");
        http::ok(self.client.request(Method::POST, Host::Default, &path)).await
    }

    /// Adds time to the opponent's clock in an ongoing game.
    /// `POST /api/round/{gameId}/add-time/{seconds}`
    pub async fn add_time(&self, game_id: &str, seconds: u32) -> Result<()> {
        let path = format!("/api/round/{game_id}/add-time/{seconds}");
        http::ok(self.client.request(Method::POST, Host::Default, &path)).await
    }

    /// Starts building an open challenge anyone can accept.
    /// `POST /api/challenge/open`
    #[must_use]
    pub fn create_open(&self) -> OpenChallengeRequest<'_> {
        OpenChallengeRequest::new(self.client)
    }

    /// Starts the clocks of a game immediately, using both players' tokens.
    /// `POST /api/challenge/{gameId}/start-clocks`
    pub async fn start_clocks(&self, game_id: &str, token1: &str, token2: &str) -> Result<()> {
        let path = format!("/api/challenge/{game_id}/start-clocks");
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .query(&[("token1", token1), ("token2", token2)]);
        http::ok(request).await
    }

    /// Creates challenge tokens for multiple users (admin only), returning a
    /// map of user id to token. `POST /api/token/admin-challenge`
    pub async fn admin_challenge_tokens(
        &self,
        user_ids: &[&str],
        description: &str,
    ) -> Result<HashMap<String, String>> {
        let request = self
            .client
            .request(Method::POST, Host::Default, "/api/token/admin-challenge")
            .form(&[
                ("users", user_ids.join(",").as_str()),
                ("description", description),
            ]);
        http::json(request, "admin challenge tokens").await
    }
}

impl LichessClient {
    /// Challenges API: create, accept, decline, and manage challenges.
    #[must_use]
    pub fn challenges(&self) -> ChallengesApi<'_> {
        ChallengesApi::new(self)
    }
}
