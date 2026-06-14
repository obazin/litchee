//! The External Engine API: register and manage external analysis engines.
//!
//! Reached through [`LichessClient::external_engine`]. The analysis/work
//! endpoints are served from `engine.lichess.ovh`.

use futures_util::stream::BoxStream;
use reqwest::header::CONTENT_TYPE;
use reqwest::{Method, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;
use crate::secret::Secret;

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
        let path = format!("/api/external-engine/{}", http::segment(id));
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
        let path = format!("/api/external-engine/{}", http::segment(id));
        let request = self
            .client
            .request(Method::PUT, Host::Default, &path)
            .json(registration);
        http::json(request, "LichessExternalEngine").await
    }

    /// Deletes an external engine. `DELETE /api/external-engine/{id}`
    pub async fn delete(&self, id: &str) -> Result<()> {
        let path = format!("/api/external-engine/{}", http::segment(id));
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
        let path = format!("/api/external-engine/{}/analyse", http::segment(id));
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
        let path = format!("/api/external-engine/work/{}", http::segment(work_id));
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

/// A registered external engine.
///
/// `client_secret` is a [`Secret`], so it is redacted from the [`Debug`] output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessExternalEngine {
    /// The unique registration id.
    pub id: String,
    /// The engine's display name.
    pub name: String,
    /// The secret used to request analysis from this engine.
    pub client_secret: Secret<String>,
    /// The user the engine is registered for.
    pub user_id: String,
    /// Maximum number of threads.
    pub max_threads: u32,
    /// Maximum hash table size, in MiB.
    pub max_hash: u32,
    /// Supported variants (UCI variant names).
    #[serde(default)]
    pub variants: Vec<String>,
    /// Arbitrary provider bookkeeping data.
    #[serde(default)]
    pub provider_data: Option<String>,
}

/// The registration body for creating or updating an external engine.
///
/// This is a request input, so it is exhaustive and constructible by callers
/// (use `..Default::default()` for the optional fields).
///
/// `provider_secret` is a [`Secret`], so it is redacted from the [`Debug`] output.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LichessExternalEngineRegistration {
    /// The engine's display name (3–200 characters).
    pub name: String,
    /// Maximum number of threads.
    pub max_threads: u32,
    /// Maximum hash table size, in MiB.
    pub max_hash: u32,
    /// A secret shared with the engine provider.
    pub provider_secret: Secret<String>,
    /// Default search depth.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_depth: Option<u32>,
    /// Supported variants (UCI variant names).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub variants: Vec<String>,
    /// Arbitrary provider bookkeeping data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_data: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_external_engine() {
        let json = r#"{"id":"eng","name":"My Engine","clientSecret":"s","userId":"u",
            "maxThreads":8,"maxHash":256,"variants":["chess"]}"#;
        let engine: LichessExternalEngine = serde_json::from_str(json).unwrap();
        assert_eq!(engine.id, "eng");
        assert_eq!(engine.max_threads, 8);
        assert_eq!(engine.variants, vec!["chess"]);
    }

    #[test]
    fn engine_debug_redacts_client_secret() {
        let engine = LichessExternalEngine {
            id: "eng".to_owned(),
            name: "My Engine".to_owned(),
            client_secret: Secret::new("supersecret".to_owned()),
            user_id: "u".to_owned(),
            max_threads: 8,
            max_hash: 256,
            variants: vec!["chess".to_owned()],
            provider_data: None,
        };
        let debug = format!("{engine:?}");
        assert!(!debug.contains("supersecret"));
        assert!(debug.contains("<redacted>"));
        assert!(debug.contains("My Engine"));
    }

    #[test]
    fn registration_debug_redacts_provider_secret() {
        let registration = LichessExternalEngineRegistration {
            name: "E".to_owned(),
            max_threads: 4,
            max_hash: 128,
            provider_secret: Secret::new("supersecret".to_owned()),
            ..Default::default()
        };
        let debug = format!("{registration:?}");
        assert!(!debug.contains("supersecret"));
        assert!(debug.contains("<redacted>"));
    }

    #[test]
    fn registration_skips_empty_optional_fields() {
        let registration = LichessExternalEngineRegistration {
            name: "E".to_owned(),
            max_threads: 4,
            max_hash: 128,
            provider_secret: Secret::new("secret".to_owned()),
            ..Default::default()
        };
        let json = serde_json::to_string(&registration).unwrap();
        assert!(!json.contains("variants"));
        assert!(!json.contains("defaultDepth"));
    }
}

/// A unit of analysis work acquired by an engine provider.
/// `POST /api/external-engine/work`
#[derive(Debug, Clone, Default, PartialEq, Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct LichessEngineWork {
    /// The work id, used to submit results.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// All other fields (the work specification), preserved verbatim.
    #[serde(flatten)]
    pub other: std::collections::HashMap<String, serde_json::Value>,
}
