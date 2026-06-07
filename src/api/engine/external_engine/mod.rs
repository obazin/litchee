//! The External Engine API: register and manage external analysis engines.
//!
//! Reached through [`LichessClient::external_engine`]. The analysis/work
//! endpoints are served from `engine.lichess.ovh`.

use futures_util::stream::BoxStream;
use reqwest::header::CONTENT_TYPE;
use reqwest::{Method, StatusCode};
use serde_json::Value;

use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;

mod model;

pub use model::{LichessEngineWork, LichessExternalEngine, LichessExternalEngineRegistration};

/// Accessor for the External Engine API.
#[derive(Debug)]
pub struct ExternalEngineApi<'a> {
    client: &'a LichessClient,
}

impl<'a> ExternalEngineApi<'a> {
    /// Binds the accessor to a client.
    pub(crate) fn new(client: &'a LichessClient) -> Self {
        Self { client }
    }

    /// Lists the authenticated user's external engines.
    /// `GET /api/external-engine`
    pub async fn list(&self) -> Result<Vec<LichessExternalEngine>> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/external-engine");
        http::json(request, "Vec<LichessExternalEngine>").await
    }

    /// Gets an external engine by id. `GET /api/external-engine/{id}`
    pub async fn get(&self, id: &str) -> Result<LichessExternalEngine> {
        let path = format!("/api/external-engine/{id}");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "LichessExternalEngine").await
    }

    /// Registers a new external engine. `POST /api/external-engine`
    pub async fn create(
        &self,
        registration: &LichessExternalEngineRegistration,
    ) -> Result<LichessExternalEngine> {
        let request = self
            .client
            .request(Method::POST, Host::Default, "/api/external-engine")
            .json(registration);
        http::json(request, "LichessExternalEngine").await
    }

    /// Updates an external engine. `PUT /api/external-engine/{id}`
    pub async fn update(
        &self,
        id: &str,
        registration: &LichessExternalEngineRegistration,
    ) -> Result<LichessExternalEngine> {
        let path = format!("/api/external-engine/{id}");
        let request = self
            .client
            .request(Method::PUT, Host::Default, &path)
            .json(registration);
        http::json(request, "LichessExternalEngine").await
    }

    /// Deletes an external engine. `DELETE /api/external-engine/{id}`
    pub async fn delete(&self, id: &str) -> Result<()> {
        let path = format!("/api/external-engine/{id}");
        http::ok(self.client.request(Method::DELETE, Host::Default, &path)).await
    }

    /// Requests analysis from an external engine, streaming the output as
    /// NDJSON. Served from `engine.lichess.ovh`.
    ///
    /// `POST /api/external-engine/{id}/analyse`
    pub async fn analyse(
        &self,
        id: &str,
        client_secret: &str,
        work: &Value,
    ) -> Result<BoxStream<'static, Result<Value>>> {
        let path = format!("/api/external-engine/{id}/analyse");
        let body = serde_json::json!({ "clientSecret": client_secret, "work": work });
        let request = self
            .client
            .request(Method::POST, Host::Engine, &path)
            .json(&body);
        http::stream(request).await
    }

    /// Acquires a unit of analysis work as an engine provider, or `None` if
    /// there is nothing to do. Served from `engine.lichess.ovh`.
    ///
    /// `POST /api/external-engine/work`
    pub async fn acquire_work(&self, provider_secret: &str) -> Result<Option<LichessEngineWork>> {
        let body = serde_json::json!({ "providerSecret": provider_secret });
        let request = self
            .client
            .request(Method::POST, Host::Engine, "/api/external-engine/work")
            .json(&body);
        let response = http::send(request).await?;
        if response.status() == StatusCode::NO_CONTENT {
            return Ok(None);
        }
        let bytes = response.bytes().await?;
        serde_json::from_slice(&bytes)
            .map(Some)
            .map_err(|err| crate::error::LichessError::decode("LichessEngineWork", err))
    }

    /// Submits engine analysis output for a unit of work. Served from
    /// `engine.lichess.ovh`.
    ///
    /// `POST /api/external-engine/work/{id}`
    pub async fn submit_work(&self, work_id: &str, output: &str) -> Result<()> {
        let path = format!("/api/external-engine/work/{work_id}");
        let request = self
            .client
            .request(Method::POST, Host::Engine, &path)
            .header(CONTENT_TYPE, "text/plain")
            .body(output.to_owned());
        http::ok(request).await
    }
}

impl LichessClient {
    /// External Engine API: register and manage external analysis engines.
    #[must_use]
    pub fn external_engine(&self) -> ExternalEngineApi<'_> {
        ExternalEngineApi::new(self)
    }
}
