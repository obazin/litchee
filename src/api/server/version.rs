//! The Server API: information about the Lichess server itself.
//!
//! Reached through [`LichessClient::server`].

use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;

/// Accessor for the Server API.
#[derive(Debug)]
pub struct ServerApi<'a> {
    client: &'a LichessClient,
}

impl<'a> ServerApi<'a> {
    /// Binds the accessor to a client.
    pub(crate) fn new(client: &'a LichessClient) -> Self {
        Self { client }
    }

    /// Gets the date and git commit of the currently deployed server code.
    ///
    /// `GET /api/version`
    ///
    /// No authentication is required.
    pub async fn version(&self) -> Result<LichessServerVersion> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/version");
        http::json(request, "LichessServerVersion").await
    }
}

impl LichessClient {
    /// Server API: information about the Lichess server itself.
    #[must_use]
    pub fn server(&self) -> ServerApi<'_> {
        ServerApi::new(self)
    }
}

/// The deployed Lichess server version.
///
/// Returned by [`ServerApi::version`] (`GET /api/version`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessServerVersion {
    /// The currently deployed server build.
    pub server: LichessServerBuild,
}

/// A single deployed build of the Lichess (`lila`) server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessServerBuild {
    /// When the deployed server was built (RFC 3339 / ISO 8601 timestamp).
    pub date: String,
    /// Git commit hash of the deployed `lila` version.
    pub commit: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_server_version() {
        let json = r#"{"server":{"date":"2026-10-28T07:25:58Z","commit":"b551752d2a0e7a63459d9812825d95a633a98737"}}"#;
        let version: LichessServerVersion = serde_json::from_str(json).unwrap();
        assert_eq!(version.server.date, "2026-10-28T07:25:58Z");
        assert_eq!(
            version.server.commit,
            "b551752d2a0e7a63459d9812825d95a633a98737"
        );
    }

    #[test]
    fn round_trips_through_json() {
        let version = LichessServerVersion {
            server: LichessServerBuild {
                date: "2026-10-28T07:25:58Z".to_owned(),
                commit: "abc123".to_owned(),
            },
        };
        let encoded = serde_json::to_string(&version).unwrap();
        let decoded: LichessServerVersion = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, version);
    }
}
