//! The Challenges API: create, accept, decline, and manage challenges.
//!
//! Reached through [`LichessClient::challenges`].

use std::collections::HashMap;

use futures_util::stream::BoxStream;
use reqwest::Method;
use reqwest::header::ACCEPT;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::gameplay::games::LichessGame;
use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;
use crate::model::{LichessColor, LichessSpeed, LichessTitle, LichessVariantKey};

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
        let path = format!("/api/challenge/{}/show", http::segment(challenge_id));
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
    ///
    /// `color` picks a side, valid only for open challenges.
    /// `POST /api/challenge/{challengeId}/accept`
    pub async fn accept(&self, challenge_id: &str, color: Option<&str>) -> Result<()> {
        let path = format!("/api/challenge/{}/accept", http::segment(challenge_id));
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .query(&[("color", color)]);
        http::ok(request).await
    }

    /// Declines an incoming challenge, optionally with a reason key.
    /// `POST /api/challenge/{challengeId}/decline`
    pub async fn decline(&self, challenge_id: &str, reason: Option<&str>) -> Result<()> {
        let path = format!("/api/challenge/{}/decline", http::segment(challenge_id));
        let mut request = self.client.request(Method::POST, Host::Default, &path);
        if let Some(reason) = reason {
            request = request.form(&[("reason", reason)]);
        }
        http::ok(request).await
    }

    /// Cancels a challenge you sent.
    ///
    /// `opponent_token` (the opponent's `challenge:write` token) lets the game
    /// be canceled even after both players moved.
    /// `POST /api/challenge/{challengeId}/cancel`
    pub async fn cancel(&self, challenge_id: &str, opponent_token: Option<&str>) -> Result<()> {
        let path = format!("/api/challenge/{}/cancel", http::segment(challenge_id));
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .query(&[("opponentToken", opponent_token)]);
        http::ok(request).await
    }

    /// Adds time to the opponent's clock in an ongoing game.
    /// `POST /api/round/{gameId}/add-time/{seconds}`
    pub async fn add_time(&self, game_id: &str, seconds: u32) -> Result<()> {
        let path = format!("/api/round/{}/add-time/{seconds}", http::segment(game_id));
        http::ok(self.client.request(Method::POST, Host::Default, &path)).await
    }

    /// Starts building an open challenge anyone can accept.
    /// `POST /api/challenge/open`
    #[must_use]
    pub fn create_open(&self) -> OpenChallengeRequest<'_> {
        OpenChallengeRequest::new(self.client)
    }

    /// Starts the clocks of a game immediately.
    ///
    /// `token2` may be omitted (`None`) for AI games that have only one player.
    /// `POST /api/challenge/{gameId}/start-clocks`
    pub async fn start_clocks(
        &self,
        game_id: &str,
        token1: &str,
        token2: Option<&str>,
    ) -> Result<()> {
        let path = format!("/api/challenge/{}/start-clocks", http::segment(game_id));
        let mut query = vec![("token1", token1)];
        if let Some(token2) = token2 {
            query.push(("token2", token2));
        }
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .query(&query);
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

/// Form body for challenging a user (`POST /api/challenge/{username}`).
#[derive(Debug, Default, Serialize)]
struct ChallengeForm<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    rated: Option<bool>,
    #[serde(rename = "clock.limit", skip_serializing_if = "Option::is_none")]
    clock_limit: Option<u32>,
    #[serde(rename = "clock.increment", skip_serializing_if = "Option::is_none")]
    clock_increment: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    days: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<LichessChallengeColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    variant: Option<LichessVariantKey>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fen: Option<&'a str>,
    #[serde(rename = "keepAliveStream", skip_serializing_if = "Option::is_none")]
    keep_alive_stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rules: Option<&'a str>,
}

/// Builder for challenging a specific user.
#[derive(Debug)]
pub struct ChallengeRequest<'a> {
    client: &'a LichessClient,
    username: &'a str,
    form: ChallengeForm<'a>,
}

impl<'a> ChallengeRequest<'a> {
    /// Creates the request builder.
    pub(crate) fn new(client: &'a LichessClient, username: &'a str) -> Self {
        Self {
            client,
            username,
            form: ChallengeForm::default(),
        }
    }

    /// Sets whether the game is rated.
    #[must_use]
    pub fn rated(mut self, rated: bool) -> Self {
        self.form.rated = Some(rated);
        self
    }

    /// Sets a real-time clock (initial seconds + increment seconds).
    #[must_use]
    pub fn clock(mut self, limit_secs: u32, increment_secs: u32) -> Self {
        self.form.clock_limit = Some(limit_secs);
        self.form.clock_increment = Some(increment_secs);
        self
    }

    /// Sets days per turn for a correspondence challenge.
    #[must_use]
    pub fn days(mut self, days: u32) -> Self {
        self.form.days = Some(days);
        self
    }

    /// Sets the requested color.
    #[must_use]
    pub fn color(mut self, color: LichessChallengeColor) -> Self {
        self.form.color = Some(color);
        self
    }

    /// Sets the variant.
    #[must_use]
    pub fn variant(mut self, variant: LichessVariantKey) -> Self {
        self.form.variant = Some(variant);
        self
    }

    /// Sets a custom starting position (FEN).
    #[must_use]
    pub fn fen(mut self, fen: &'a str) -> Self {
        self.form.fen = Some(fen);
        self
    }

    /// Sets extra game rules (comma-separated, e.g. `noRematch,noGiveTime`).
    #[must_use]
    pub fn rules(mut self, rules: &'a str) -> Self {
        self.form.rules = Some(rules);
        self
    }

    /// Sends the challenge, returning it immediately.
    pub async fn send(self) -> Result<LichessChallenge> {
        let request = self.request();
        http::json(request, "LichessChallenge").await
    }

    /// Sends the challenge with `keepAliveStream`, returning an NDJSON stream
    /// that is held open and emits status updates (e.g. `{"done":"accepted"}`)
    /// until the challenge is accepted, declined, or canceled.
    pub async fn stream(mut self) -> Result<BoxStream<'static, Result<Value>>> {
        self.form.keep_alive_stream = Some(true);
        let request = self.request().header(ACCEPT, http::ACCEPT_NDJSON);
        http::stream(request, self.client.max_line_bytes()).await
    }

    /// Builds the POST request for the challenge form.
    fn request(&self) -> http::ApiRequest {
        let path = format!("/api/challenge/{}", http::segment(self.username));
        self.client
            .request(Method::POST, Host::Default, &path)
            .form(&self.form)
    }
}

/// Form body for challenging the AI (`POST /api/challenge/ai`).
#[derive(Debug, Serialize)]
struct AiChallengeForm<'a> {
    level: u8,
    #[serde(rename = "clock.limit", skip_serializing_if = "Option::is_none")]
    clock_limit: Option<u32>,
    #[serde(rename = "clock.increment", skip_serializing_if = "Option::is_none")]
    clock_increment: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    days: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<LichessChallengeColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    variant: Option<LichessVariantKey>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fen: Option<&'a str>,
}

/// Builder for challenging the Lichess AI.
#[derive(Debug)]
pub struct AiChallengeRequest<'a> {
    client: &'a LichessClient,
    form: AiChallengeForm<'a>,
}

impl<'a> AiChallengeRequest<'a> {
    /// Creates the request builder.
    pub(crate) fn new(client: &'a LichessClient, level: u8) -> Self {
        Self {
            client,
            form: AiChallengeForm {
                level,
                clock_limit: None,
                clock_increment: None,
                days: None,
                color: None,
                variant: None,
                fen: None,
            },
        }
    }

    /// Sets a real-time clock (initial seconds + increment seconds).
    #[must_use]
    pub fn clock(mut self, limit_secs: u32, increment_secs: u32) -> Self {
        self.form.clock_limit = Some(limit_secs);
        self.form.clock_increment = Some(increment_secs);
        self
    }

    /// Sets days per turn for a correspondence game.
    #[must_use]
    pub fn days(mut self, days: u32) -> Self {
        self.form.days = Some(days);
        self
    }

    /// Sets the color the authenticated user plays.
    #[must_use]
    pub fn color(mut self, color: LichessChallengeColor) -> Self {
        self.form.color = Some(color);
        self
    }

    /// Sets the variant.
    #[must_use]
    pub fn variant(mut self, variant: LichessVariantKey) -> Self {
        self.form.variant = Some(variant);
        self
    }

    /// Sets a custom starting position (FEN).
    #[must_use]
    pub fn fen(mut self, fen: &'a str) -> Self {
        self.form.fen = Some(fen);
        self
    }

    /// Starts the game against the AI.
    pub async fn send(self) -> Result<LichessGame> {
        let request = self
            .client
            .request(Method::POST, Host::Default, "/api/challenge/ai")
            .form(&self.form);
        http::json(request, "LichessGame").await
    }
}

/// Form body for an open challenge (`POST /api/challenge/open`).
#[derive(Debug, Default, Serialize)]
struct OpenForm<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    rated: Option<bool>,
    #[serde(rename = "clock.limit", skip_serializing_if = "Option::is_none")]
    clock_limit: Option<u32>,
    #[serde(rename = "clock.increment", skip_serializing_if = "Option::is_none")]
    clock_increment: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    days: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    variant: Option<LichessVariantKey>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fen: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rules: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    users: Option<&'a str>,
    #[serde(rename = "expiresAt", skip_serializing_if = "Option::is_none")]
    expires_at: Option<i64>,
}

/// Builder for an open challenge that anyone can accept.
#[derive(Debug)]
pub struct OpenChallengeRequest<'a> {
    client: &'a LichessClient,
    form: OpenForm<'a>,
}

impl<'a> OpenChallengeRequest<'a> {
    /// Creates the request builder.
    pub(crate) fn new(client: &'a LichessClient) -> Self {
        Self {
            client,
            form: OpenForm::default(),
        }
    }

    /// Sets whether the game is rated.
    #[must_use]
    pub fn rated(mut self, rated: bool) -> Self {
        self.form.rated = Some(rated);
        self
    }

    /// Sets a real-time clock (initial seconds + increment seconds).
    #[must_use]
    pub fn clock(mut self, limit_secs: u32, increment_secs: u32) -> Self {
        self.form.clock_limit = Some(limit_secs);
        self.form.clock_increment = Some(increment_secs);
        self
    }

    /// Sets days per turn for a correspondence challenge.
    #[must_use]
    pub fn days(mut self, days: u32) -> Self {
        self.form.days = Some(days);
        self
    }

    /// Sets the variant.
    #[must_use]
    pub fn variant(mut self, variant: LichessVariantKey) -> Self {
        self.form.variant = Some(variant);
        self
    }

    /// Sets an optional name shown on the challenge page.
    #[must_use]
    pub fn name(mut self, name: &'a str) -> Self {
        self.form.name = Some(name);
        self
    }

    /// Sets a custom starting position (FEN).
    #[must_use]
    pub fn fen(mut self, fen: &'a str) -> Self {
        self.form.fen = Some(fen);
        self
    }

    /// Sets extra game rules (comma-separated, e.g. `noRematch,noGiveTime`).
    #[must_use]
    pub fn rules(mut self, rules: &'a str) -> Self {
        self.form.rules = Some(rules);
        self
    }

    /// Restricts who may play to these two usernames (comma-separated); the
    /// game is then created between them.
    #[must_use]
    pub fn users(mut self, users: &'a str) -> Self {
        self.form.users = Some(users);
        self
    }

    /// Sets when the challenge expires (Unix milliseconds).
    #[must_use]
    pub fn expires_at(mut self, timestamp: i64) -> Self {
        self.form.expires_at = Some(timestamp);
        self
    }

    /// Creates the open challenge.
    pub async fn send(self) -> Result<LichessOpenChallenge> {
        let request = self
            .client
            .request(Method::POST, Host::Default, "/api/challenge/open")
            .form(&self.form);
        http::json(request, "LichessOpenChallenge").await
    }
}

/// The lifecycle status of a challenge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum LichessChallengeStatus {
    /// Created and pending.
    Created,
    /// The destination user is offline.
    Offline,
    /// Canceled by the challenger.
    Canceled,
    /// Declined by the destination user.
    Declined,
    /// Accepted; a game has started.
    Accepted,
}

/// The requested color of a challenge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum LichessChallengeColor {
    /// Play as white.
    White,
    /// Play as black.
    Black,
    /// Random color.
    Random,
}

/// A user involved in a challenge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessChallengeUser {
    /// The user id.
    pub id: String,
    /// The display name.
    pub name: String,
    /// The user's rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rating: Option<u32>,
    /// The user's title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<LichessTitle>,
    /// Whether the rating is provisional.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provisional: Option<bool>,
    /// Whether the user is online.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub online: Option<bool>,
    /// The user's network lag in milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lag: Option<u32>,
}

/// The time control of a challenge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
#[non_exhaustive]
pub enum LichessTimeControl {
    /// A real-time clock.
    Clock {
        /// Initial time in seconds.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        limit: Option<u32>,
        /// Increment per move in seconds.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        increment: Option<u32>,
        /// Human-readable form (e.g. `"5+2"`).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        show: Option<String>,
    },
    /// A correspondence time control.
    Correspondence {
        /// Days per turn.
        #[serde(
            rename = "daysPerTurn",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        days_per_turn: Option<u32>,
    },
    /// No time limit.
    Unlimited,
}

/// Display info for a challenge's perf.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessChallengePerf {
    /// The perf icon.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// The perf name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Variant details for a challenge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessChallengeVariant {
    /// The variant key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<LichessVariantKey>,
    /// The full variant name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// A short variant name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub short: Option<String>,
}

/// A challenge between two players.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessChallenge {
    /// The challenge id.
    pub id: String,
    /// The challenge URL.
    pub url: String,
    /// The status.
    pub status: LichessChallengeStatus,
    /// The challenger (absent for open challenges).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub challenger: Option<LichessChallengeUser>,
    /// The destination user (absent for open challenges).
    #[serde(default)]
    pub dest_user: Option<LichessChallengeUser>,
    /// The variant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<LichessChallengeVariant>,
    /// Whether the game would be rated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rated: Option<bool>,
    /// The speed category.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed: Option<LichessSpeed>,
    /// The time control.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_control: Option<LichessTimeControl>,
    /// The requested color.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<LichessChallengeColor>,
    /// The resolved color, if random has been settled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub final_color: Option<LichessColor>,
    /// Perf display info.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub perf: Option<LichessChallengePerf>,
    /// The direction (`in` or `out`) relative to the authenticated user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    /// The initial FEN, for custom positions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_fen: Option<String>,
    /// The id of the game this is a rematch of.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rematch_of: Option<String>,
}

/// The incoming and outgoing challenges for the authenticated user.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessChallenges {
    /// Challenges sent to the user.
    #[serde(rename = "in", default)]
    pub incoming: Vec<LichessChallenge>,
    /// Challenges sent by the user.
    #[serde(rename = "out", default)]
    pub outgoing: Vec<LichessChallenge>,
}

/// An open challenge that anyone can accept.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessOpenChallenge {
    /// The challenge id.
    pub id: String,
    /// The challenge URL.
    pub url: String,
    /// The status.
    pub status: LichessChallengeStatus,
    /// Whether the game would be rated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rated: Option<bool>,
    /// The variant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<LichessChallengeVariant>,
    /// The time control.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_control: Option<LichessTimeControl>,
    /// URL for the player taking white.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url_white: Option<String>,
    /// URL for the player taking black.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url_black: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_challenge_with_clock_time_control() {
        let json = r#"{"id":"H9fIRZUk","url":"https://lichess.org/H9fIRZUk",
            "status":"created","challenger":{"id":"bot1","name":"Bot1","rating":1500},
            "destUser":{"id":"bobby","name":"Bobby"},"rated":true,"speed":"rapid",
            "timeControl":{"type":"clock","limit":600,"increment":0,"show":"10+0"},
            "color":"random","finalColor":"black","direction":"out"}"#;
        let challenge: LichessChallenge = serde_json::from_str(json).unwrap();
        assert_eq!(challenge.id, "H9fIRZUk");
        assert_eq!(challenge.color, Some(LichessChallengeColor::Random));
        assert_eq!(
            challenge.time_control,
            Some(LichessTimeControl::Clock {
                limit: Some(600),
                increment: Some(0),
                show: Some("10+0".to_owned()),
            })
        );
    }

    #[test]
    fn parses_challenge_list_in_out() {
        let json = r#"{"in":[],"out":[{"id":"x","url":"u","status":"created"}]}"#;
        let challenges: LichessChallenges = serde_json::from_str(json).unwrap();
        assert!(challenges.incoming.is_empty());
        assert_eq!(challenges.outgoing.len(), 1);
    }
}
