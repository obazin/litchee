//! The Bulk Pairing API: create and manage batches of games.
//!
//! Reached through [`LichessClient::bulk_pairing`].

use futures_util::stream::BoxStream;
use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::api::gameplay::games::LichessGame;
use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;
use crate::model::{GameExportOptions, LichessVariantKey};

/// Accessor for the Bulk Pairing API.
#[derive(Debug)]
pub struct BulkPairingApi<'a> {
    client: &'a LichessClient,
}

impl<'a> BulkPairingApi<'a> {
    /// Binds the accessor to a client.
    pub(crate) fn new(client: &'a LichessClient) -> Self {
        Self { client }
    }

    /// Lists the authenticated user's bulk pairings. `GET /api/bulk-pairing`
    pub async fn list(&self) -> Result<Vec<LichessBulkPairing>> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/bulk-pairing");
        http::json(request, "Vec<LichessBulkPairing>").await
    }

    /// Gets a bulk pairing by id. `GET /api/bulk-pairing/{id}`
    pub async fn get(&self, id: &str) -> Result<LichessBulkPairing> {
        let path = format!("/api/bulk-pairing/{}", http::segment(id));
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "LichessBulkPairing").await
    }

    /// Starts building a bulk pairing from a `players` token-pair string
    /// (e.g. `"token1:token2,token3:token4"`). `POST /api/bulk-pairing`
    #[must_use]
    pub fn create(&self, players: &'a str) -> BulkPairingRequest<'a> {
        BulkPairingRequest::new(self.client, players)
    }

    /// Immediately starts the clocks of a bulk pairing.
    /// `POST /api/bulk-pairing/{id}/start-clocks`
    pub async fn start_clocks(&self, id: &str) -> Result<()> {
        let path = format!("/api/bulk-pairing/{}/start-clocks", http::segment(id));
        http::ok(self.client.request(Method::POST, Host::Default, &path)).await
    }

    /// Cancels (deletes) a bulk pairing that has not started.
    /// `DELETE /api/bulk-pairing/{id}`
    pub async fn delete(&self, id: &str) -> Result<()> {
        let path = format!("/api/bulk-pairing/{}", http::segment(id));
        http::ok(self.client.request(Method::DELETE, Host::Default, &path)).await
    }

    /// Starts an export of a bulk pairing's games.
    /// `GET /api/bulk-pairing/{id}/games`
    ///
    /// Finish with [`stream`](BulkGamesRequest::stream) or
    /// [`pgn`](BulkGamesRequest::pgn).
    #[must_use]
    pub fn games(&self, id: &'a str) -> BulkGamesRequest<'a> {
        BulkGamesRequest::new(self.client, id)
    }
}

/// Builder for exporting a bulk pairing's games
/// (`GET /api/bulk-pairing/{id}/games`).
#[derive(Debug)]
pub struct BulkGamesRequest<'a> {
    client: &'a LichessClient,
    id: &'a str,
    export: GameExportOptions,
}

impl<'a> BulkGamesRequest<'a> {
    /// Creates the request builder.
    pub(crate) fn new(client: &'a LichessClient, id: &'a str) -> Self {
        Self {
            client,
            id,
            export: GameExportOptions::default(),
        }
    }

    /// Sets the shared export-format options (moves, clocks, evals, …).
    #[must_use]
    pub fn export(mut self, options: GameExportOptions) -> Self {
        self.export = options;
        self
    }

    /// Executes the export, streaming games as decoded JSON values.
    pub async fn stream(self) -> Result<BoxStream<'static, Result<LichessGame>>> {
        let request = self.request(http::ACCEPT_NDJSON);
        http::stream(request, self.client.max_line_bytes()).await
    }

    /// Executes the export, returning all games as one PGN string.
    pub async fn pgn(self) -> Result<String> {
        http::text(self.request(http::ACCEPT_PGN)).await
    }

    /// Builds the request with the given `Accept` representation.
    fn request(&self, accept: &'static str) -> http::ApiRequest {
        let path = format!("/api/bulk-pairing/{}/games", http::segment(self.id));
        self.client
            .request(Method::GET, Host::Default, &path)
            .header(reqwest::header::ACCEPT, accept)
            .query(&self.export)
    }
}

/// Form body for creating a bulk pairing.
#[derive(Debug, Serialize)]
struct BulkPairingForm<'a> {
    players: &'a str,
    #[serde(rename = "clock.limit", skip_serializing_if = "Option::is_none")]
    clock_limit: Option<u32>,
    #[serde(rename = "clock.increment", skip_serializing_if = "Option::is_none")]
    clock_increment: Option<u32>,
    #[serde(rename = "pairAt", skip_serializing_if = "Option::is_none")]
    pair_at: Option<i64>,
    #[serde(rename = "startClocksAt", skip_serializing_if = "Option::is_none")]
    start_clocks_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    variant: Option<LichessVariantKey>,
}

/// Builder for creating a bulk pairing.
#[derive(Debug)]
pub struct BulkPairingRequest<'a> {
    client: &'a LichessClient,
    form: BulkPairingForm<'a>,
}

impl<'a> BulkPairingRequest<'a> {
    /// Creates the request builder.
    fn new(client: &'a LichessClient, players: &'a str) -> Self {
        Self {
            client,
            form: BulkPairingForm {
                players,
                clock_limit: None,
                clock_increment: None,
                pair_at: None,
                start_clocks_at: None,
                rated: None,
                variant: None,
            },
        }
    }

    /// Sets the clock (initial seconds + increment seconds).
    #[must_use]
    pub fn clock(mut self, limit_secs: u32, increment_secs: u32) -> Self {
        self.form.clock_limit = Some(limit_secs);
        self.form.clock_increment = Some(increment_secs);
        self
    }

    /// Sets whether the games are rated.
    #[must_use]
    pub fn rated(mut self, rated: bool) -> Self {
        self.form.rated = Some(rated);
        self
    }

    /// Sets when the games are created (Unix milliseconds).
    #[must_use]
    pub fn pair_at(mut self, timestamp: i64) -> Self {
        self.form.pair_at = Some(timestamp);
        self
    }

    /// Sets when the clocks start (Unix milliseconds).
    #[must_use]
    pub fn start_clocks_at(mut self, timestamp: i64) -> Self {
        self.form.start_clocks_at = Some(timestamp);
        self
    }

    /// Sets the variant.
    #[must_use]
    pub fn variant(mut self, variant: LichessVariantKey) -> Self {
        self.form.variant = Some(variant);
        self
    }

    /// Creates the bulk pairing.
    pub async fn send(self) -> Result<LichessBulkPairing> {
        let request = self
            .client
            .request(Method::POST, Host::Default, "/api/bulk-pairing")
            .form(&self.form);
        http::json(request, "LichessBulkPairing").await
    }
}

impl LichessClient {
    /// Bulk Pairing API: create and manage batches of games.
    #[must_use]
    pub fn bulk_pairing(&self) -> BulkPairingApi<'_> {
        BulkPairingApi::new(self)
    }
}

/// A clock as used by bulk pairings (`limit` + `increment`, in seconds).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessBulkClock {
    /// Initial time in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Increment per move in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub increment: Option<u32>,
}

/// A single pairing within a bulk pairing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessBulkPairingGame {
    /// The game id.
    pub id: String,
    /// The white player's id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub white: Option<String>,
    /// The black player's id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub black: Option<String>,
}

/// A bulk pairing: a batch of games created together.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessBulkPairing {
    /// The bulk pairing id.
    pub id: String,
    /// The paired games.
    #[serde(default)]
    pub games: Vec<LichessBulkPairingGame>,
    /// The variant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<LichessVariantKey>,
    /// The clock.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clock: Option<LichessBulkClock>,
    /// When the games will be created (Unix milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pair_at: Option<i64>,
    /// When the games were created, if already paired.
    #[serde(default)]
    pub paired_at: Option<i64>,
    /// Whether the games are rated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rated: Option<bool>,
    /// When the clocks will start (Unix milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_clocks_at: Option<i64>,
    /// When the pairing was scheduled (Unix milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scheduled_at: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bulk_pairing_with_null_paired_at() {
        let json = r#"{"id":"RVAcwgg7",
            "games":[{"id":"NKop9IyD","black":"lizen1","white":"thibault"}],
            "variant":"standard","clock":{"increment":0,"limit":300},
            "pairAt":1612289869919,"pairedAt":null,"rated":false,
            "startClocksAt":1612200422971,"scheduledAt":1612203514628}"#;
        let pairing: LichessBulkPairing = serde_json::from_str(json).unwrap();
        assert_eq!(pairing.games.len(), 1);
        assert_eq!(pairing.paired_at, None);
        assert_eq!(pairing.clock.unwrap().limit, Some(300));
    }
}
