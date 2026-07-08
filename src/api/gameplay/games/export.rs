//! Builders for the game-export endpoints (single game and a user's games).

use futures_util::stream::BoxStream;
use reqwest::Method;
use reqwest::header::ACCEPT;
use serde::Serialize;

use super::model::LichessGame;
use super::{NDJSON, PGN};
use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;

/// Builder for exporting a single game (`GET /game/export/{gameId}`).
#[derive(Debug)]
pub struct GameExportRequest<'a> {
    client: &'a LichessClient,
    game_id: &'a str,
    query: SingleExportQuery,
}

/// Query options shared by both export endpoints' formatting.
#[derive(Debug, Default, Serialize)]
struct SingleExportQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    moves: Option<bool>,
    #[serde(rename = "pgnInJson", skip_serializing_if = "Option::is_none")]
    pgn_in_json: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    clocks: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    evals: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    accuracy: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    opening: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    literate: Option<bool>,
}

impl<'a> GameExportRequest<'a> {
    /// Creates the request builder.
    pub(crate) fn new(client: &'a LichessClient, game_id: &'a str) -> Self {
        Self {
            client,
            game_id,
            query: SingleExportQuery::default(),
        }
    }

    /// Includes the full PGN inside the JSON response (`pgn` field).
    #[must_use]
    pub fn pgn_in_json(mut self, value: bool) -> Self {
        self.query.pgn_in_json = Some(value);
        self
    }

    /// Includes clock comments / fields.
    #[must_use]
    pub fn clocks(mut self, value: bool) -> Self {
        self.query.clocks = Some(value);
        self
    }

    /// Includes analysis evaluations.
    #[must_use]
    pub fn evals(mut self, value: bool) -> Self {
        self.query.evals = Some(value);
        self
    }

    /// Includes the opening name.
    #[must_use]
    pub fn opening(mut self, value: bool) -> Self {
        self.query.opening = Some(value);
        self
    }

    /// Includes per-player accuracy (JSON only).
    #[must_use]
    pub fn accuracy(mut self, value: bool) -> Self {
        self.query.accuracy = Some(value);
        self
    }

    /// Executes the export, returning the game as JSON.
    pub async fn json(self) -> Result<LichessGame> {
        let path = format!("/game/export/{}", http::segment(self.game_id));
        let request = self
            .client
            .request(Method::GET, Host::Default, &path)
            .query(&self.query);
        http::json(request, "LichessGame").await
    }

    /// Executes the export, returning the game as a PGN string.
    pub async fn pgn(self) -> Result<String> {
        let path = format!("/game/export/{}", http::segment(self.game_id));
        let request = self
            .client
            .request(Method::GET, Host::Default, &path)
            .header(ACCEPT, PGN)
            .query(&self.query);
        http::text(request).await
    }
}

/// Builder for exporting a user's games (`GET /api/games/user/{username}`).
#[derive(Debug)]
pub struct UserGamesRequest<'a> {
    client: &'a LichessClient,
    username: &'a str,
    query: UserGamesQuery<'a>,
}

/// Query parameters for exporting a user's games.
#[derive(Debug, Default, Serialize)]
struct UserGamesQuery<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    max: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    since: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    until: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vs: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rated: Option<bool>,
    #[serde(rename = "perfType", skip_serializing_if = "Option::is_none")]
    perf_type: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    analysed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ongoing: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    finished: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    opening: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    evals: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    clocks: Option<bool>,
}

impl<'a> UserGamesRequest<'a> {
    /// Creates the request builder.
    pub(crate) fn new(client: &'a LichessClient, username: &'a str) -> Self {
        Self {
            client,
            username,
            query: UserGamesQuery::default(),
        }
    }

    /// Limits the number of games downloaded.
    #[must_use]
    pub fn max(mut self, count: u32) -> Self {
        self.query.max = Some(count);
        self
    }

    /// Only games played since this timestamp (Unix milliseconds).
    #[must_use]
    pub fn since(mut self, timestamp: i64) -> Self {
        self.query.since = Some(timestamp);
        self
    }

    /// Only games played until this timestamp (Unix milliseconds).
    #[must_use]
    pub fn until(mut self, timestamp: i64) -> Self {
        self.query.until = Some(timestamp);
        self
    }

    /// Only rated (`true`) or casual (`false`) games.
    #[must_use]
    pub fn rated(mut self, rated: bool) -> Self {
        self.query.rated = Some(rated);
        self
    }

    /// Only games in these speeds/variants (comma-separated perf types).
    #[must_use]
    pub fn perf_type(mut self, perf_type: &'a str) -> Self {
        self.query.perf_type = Some(perf_type);
        self
    }

    /// Only games played as this color (`"white"` or `"black"`).
    #[must_use]
    pub fn color(mut self, color: &'a str) -> Self {
        self.query.color = Some(color);
        self
    }

    /// Only currently-ongoing games.
    #[must_use]
    pub fn ongoing(mut self, ongoing: bool) -> Self {
        self.query.ongoing = Some(ongoing);
        self
    }

    /// Executes the export, streaming games as decoded JSON values.
    pub async fn stream(self) -> Result<BoxStream<'static, Result<LichessGame>>> {
        let path = format!("/api/games/user/{}", http::segment(self.username));
        let request = self
            .client
            .request(Method::GET, Host::Default, &path)
            .header(ACCEPT, NDJSON)
            .query(&self.query);
        http::stream(request, self.client.max_line_bytes()).await
    }

    /// Executes the export, returning all games as one PGN string.
    pub async fn pgn(self) -> Result<String> {
        let path = format!("/api/games/user/{}", http::segment(self.username));
        let request = self
            .client
            .request(Method::GET, Host::Default, &path)
            .header(ACCEPT, PGN)
            .query(&self.query);
        http::text(request).await
    }
}
