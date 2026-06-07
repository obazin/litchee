//! The FIDE API: look up FIDE-rated players and their rating histories.
//!
//! Reached through [`LichessClient::fide`].

use reqwest::Method;

use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;

mod model;

pub use model::{LichessFideGender, LichessFidePhoto, LichessFidePlayer, LichessFidePlayerRatings};

/// Accessor for the FIDE API.
#[derive(Debug)]
pub struct FideApi<'a> {
    client: &'a LichessClient,
}

impl<'a> FideApi<'a> {
    /// Binds the accessor to a client.
    pub(crate) fn new(client: &'a LichessClient) -> Self {
        Self { client }
    }

    /// Gets a FIDE player by id.
    ///
    /// `GET /api/fide/player/{playerId}`
    pub async fn get(&self, player_id: u32) -> Result<LichessFidePlayer> {
        let path = format!("/api/fide/player/{player_id}");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "LichessFidePlayer").await
    }

    /// Gets a FIDE player's encoded rating histories.
    ///
    /// `GET /api/fide/player/{playerId}/ratings`
    pub async fn ratings(&self, player_id: u32) -> Result<LichessFidePlayerRatings> {
        let path = format!("/api/fide/player/{player_id}/ratings");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "LichessFidePlayerRatings").await
    }

    /// Searches FIDE players by query.
    ///
    /// `GET /api/fide/player`
    pub async fn search(&self, query: &str) -> Result<Vec<LichessFidePlayer>> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/fide/player")
            .query(&[("q", query)]);
        http::json(request, "Vec<LichessFidePlayer>").await
    }
}

impl LichessClient {
    /// FIDE API: look up FIDE-rated players and their rating histories.
    #[must_use]
    pub fn fide(&self) -> FideApi<'_> {
        FideApi::new(self)
    }
}
