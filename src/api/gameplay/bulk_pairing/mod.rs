//! The Bulk Pairing API: create and manage batches of games.
//!
//! Reached through [`LichessClient::bulk_pairing`].

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

pub use model::{LichessBulkClock, LichessBulkPairing, LichessBulkPairingGame};

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
        let path = format!("/api/bulk-pairing/{id}");
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
        let path = format!("/api/bulk-pairing/{id}/start-clocks");
        http::ok(self.client.request(Method::POST, Host::Default, &path)).await
    }

    /// Cancels (deletes) a bulk pairing that has not started.
    /// `DELETE /api/bulk-pairing/{id}`
    pub async fn delete(&self, id: &str) -> Result<()> {
        let path = format!("/api/bulk-pairing/{id}");
        http::ok(self.client.request(Method::DELETE, Host::Default, &path)).await
    }

    /// Streams the games of a bulk pairing as NDJSON.
    /// `GET /api/bulk-pairing/{id}/games`
    pub async fn games(&self, id: &str) -> Result<BoxStream<'static, Result<LichessGame>>> {
        let path = format!("/api/bulk-pairing/{id}/games");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::stream(request).await
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
