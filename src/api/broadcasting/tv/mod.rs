//! The TV API: featured games and live feeds.
//!
//! Reached through [`LichessClient::tv`].

use futures_util::stream::BoxStream;
use reqwest::Method;
use reqwest::header::ACCEPT;

use crate::api::gameplay::games::LichessGame;
use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;

mod model;

pub use model::{
    LichessTvChannels, LichessTvFeatured, LichessTvFeedEvent, LichessTvFeedPlayer, LichessTvGame,
    LichessTvMove,
};

/// The `application/x-ndjson` content type.
const NDJSON: &str = "application/x-ndjson";

/// Accessor for the TV API.
#[derive(Debug)]
pub struct TvApi<'a> {
    client: &'a LichessClient,
}

impl<'a> TvApi<'a> {
    /// Binds the accessor to a client.
    pub(crate) fn new(client: &'a LichessClient) -> Self {
        Self { client }
    }

    /// Gets the current best game for each channel. `GET /api/tv/channels`
    pub async fn channels(&self) -> Result<LichessTvChannels> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/tv/channels");
        http::json(request, "LichessTvChannels").await
    }

    /// Streams the feed of the current overall featured game.
    ///
    /// `GET /api/tv/feed`
    pub async fn feed(&self) -> Result<BoxStream<'static, Result<LichessTvFeedEvent>>> {
        self.feed_at("/api/tv/feed").await
    }

    /// Streams the feed of a specific channel's featured game.
    ///
    /// `GET /api/tv/{channel}/feed`
    pub async fn channel_feed(
        &self,
        channel: &str,
    ) -> Result<BoxStream<'static, Result<LichessTvFeedEvent>>> {
        let path = format!("/api/tv/{channel}/feed");
        self.feed_at(&path).await
    }

    /// Streams the best ongoing games of a channel as NDJSON.
    ///
    /// `GET /api/tv/{channel}`
    pub async fn channel_games(
        &self,
        channel: &str,
        nb: u32,
    ) -> Result<BoxStream<'static, Result<LichessGame>>> {
        let path = format!("/api/tv/{channel}");
        let request = self
            .client
            .request(Method::GET, Host::Default, &path)
            .header(ACCEPT, NDJSON)
            .query(&[("nb", nb)]);
        http::stream(request).await
    }

    /// Opens a TV feed stream at the given path.
    async fn feed_at(&self, path: &str) -> Result<BoxStream<'static, Result<LichessTvFeedEvent>>> {
        let request = self.client.request(Method::GET, Host::Default, path);
        http::stream(request).await
    }
}

impl LichessClient {
    /// TV API: featured games and live feeds.
    #[must_use]
    pub fn tv(&self) -> TvApi<'_> {
        TvApi::new(self)
    }
}
