//! Builders for creating challenges.

use reqwest::Method;
use serde::Serialize;

use super::model::{LichessChallenge, LichessChallengeColor, LichessOpenChallenge};
use crate::api::gameplay::games::LichessGame;
use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;
use crate::model::LichessVariantKey;

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

    /// Sends the challenge.
    pub async fn send(self) -> Result<LichessChallenge> {
        let path = format!("/api/challenge/{}", self.username);
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .form(&self.form);
        http::json(request, "LichessChallenge").await
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

    /// Creates the open challenge.
    pub async fn send(self) -> Result<LichessOpenChallenge> {
        let request = self
            .client
            .request(Method::POST, Host::Default, "/api/challenge/open")
            .form(&self.form);
        http::json(request, "LichessOpenChallenge").await
    }
}
