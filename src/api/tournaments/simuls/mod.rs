//! The Simuls API: simultaneous exhibitions.
//!
//! Reached through [`LichessClient::simuls`].

use reqwest::Method;

use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;

mod model;

pub use model::{LichessSimul, LichessSimulHost, LichessSimulVariant, LichessSimuls};

/// Accessor for the Simuls API.
#[derive(Debug)]
pub struct SimulsApi<'a> {
    client: &'a LichessClient,
}

impl<'a> SimulsApi<'a> {
    /// Binds the accessor to a client.
    pub(crate) fn new(client: &'a LichessClient) -> Self {
        Self { client }
    }

    /// Gets the current simuls, grouped by lifecycle stage.
    ///
    /// When authenticated, `pending` includes your created-but-unstarted
    /// simuls. `GET /api/simul`
    pub async fn current(&self) -> Result<LichessSimuls> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/simul");
        http::json(request, "LichessSimuls").await
    }
}

impl LichessClient {
    /// Simuls API: simultaneous exhibitions.
    #[must_use]
    pub fn simuls(&self) -> SimulsApi<'_> {
        SimulsApi::new(self)
    }
}
