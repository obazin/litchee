//! The Tablebase API: endgame tablebase lookups.
//!
//! Served from `tablebase.lichess.org`. Reached through
//! [`LichessClient::tablebase`].

use reqwest::Method;

use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;

mod model;

pub use model::{LichessTablebaseCategory, LichessTablebaseMove, LichessTablebasePosition};

/// Accessor for the Tablebase API.
#[derive(Debug)]
pub struct TablebaseApi<'a> {
    client: &'a LichessClient,
}

impl<'a> TablebaseApi<'a> {
    /// Binds the accessor to a client.
    pub(crate) fn new(client: &'a LichessClient) -> Self {
        Self { client }
    }

    /// Looks up a standard-chess position. `GET /standard`
    pub async fn standard(&self, fen: &str) -> Result<LichessTablebasePosition> {
        self.lookup("/standard", fen).await
    }

    /// Looks up an atomic-chess position. `GET /atomic`
    pub async fn atomic(&self, fen: &str) -> Result<LichessTablebasePosition> {
        self.lookup("/atomic", fen).await
    }

    /// Looks up an antichess position. `GET /antichess`
    pub async fn antichess(&self, fen: &str) -> Result<LichessTablebasePosition> {
        self.lookup("/antichess", fen).await
    }

    /// Issues a tablebase lookup against the tablebase host.
    async fn lookup(&self, path: &str, fen: &str) -> Result<LichessTablebasePosition> {
        let request = self
            .client
            .request(Method::GET, Host::Tablebase, path)
            .query(&[("fen", fen)]);
        http::json(request, "LichessTablebasePosition").await
    }
}

impl LichessClient {
    /// Tablebase API: endgame tablebase lookups (`tablebase.lichess.org`).
    #[must_use]
    pub fn tablebase(&self) -> TablebaseApi<'_> {
        TablebaseApi::new(self)
    }
}
