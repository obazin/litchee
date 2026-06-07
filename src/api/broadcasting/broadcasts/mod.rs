//! The Broadcasts API: official broadcasts, rounds, players, and PGN.
//!
//! Reached through [`LichessClient::broadcasts`].

use futures_util::stream::BoxStream;
use reqwest::Method;
use reqwest::header::CONTENT_TYPE;

use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;

mod manage;
mod model;

pub use manage::{RoundRequest, TourRequest};
pub use model::{
    LichessBroadcast, LichessBroadcastMyRound, LichessBroadcastPlayerEntry,
    LichessBroadcastPushResult, LichessBroadcastRoundInfo, LichessBroadcastRoundView,
    LichessBroadcastSearchPage, LichessBroadcastTop, LichessBroadcastTour,
};

/// Accessor for the Broadcasts API.
#[derive(Debug)]
pub struct BroadcastsApi<'a> {
    client: &'a LichessClient,
}

impl<'a> BroadcastsApi<'a> {
    /// Binds the accessor to a client.
    pub(crate) fn new(client: &'a LichessClient) -> Self {
        Self { client }
    }

    /// Streams official broadcasts. `GET /api/broadcast`
    pub async fn official(&self) -> Result<BoxStream<'static, Result<LichessBroadcast>>> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/broadcast");
        http::stream(request).await
    }

    /// Gets the top broadcasts (active, upcoming, past). `GET /api/broadcast/top`
    pub async fn top(&self) -> Result<LichessBroadcastTop> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/broadcast/top");
        http::json(request, "LichessBroadcastTop").await
    }

    /// Searches broadcasts. `GET /api/broadcast/search`
    pub async fn search(&self, query: &str, page: u32) -> Result<LichessBroadcastSearchPage> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/broadcast/search")
            .query(&[("q", query), ("page", &page.to_string())]);
        http::json(request, "LichessBroadcastSearchPage").await
    }

    /// Streams broadcasts created by a user. `GET /api/broadcast/by/{username}`
    pub async fn by_user(
        &self,
        username: &str,
    ) -> Result<BoxStream<'static, Result<LichessBroadcast>>> {
        let path = format!("/api/broadcast/by/{username}");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::stream(request).await
    }

    /// Streams the authenticated user's broadcast rounds.
    /// `GET /api/broadcast/my-rounds`
    pub async fn my_rounds(&self) -> Result<BoxStream<'static, Result<LichessBroadcastMyRound>>> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/broadcast/my-rounds");
        http::stream(request).await
    }

    /// Gets a broadcast tournament with its rounds.
    /// `GET /api/broadcast/{broadcastTournamentId}`
    pub async fn get_tournament(&self, tournament_id: &str) -> Result<LichessBroadcast> {
        let path = format!("/api/broadcast/{tournament_id}");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "LichessBroadcast").await
    }

    /// Gets a round with its games.
    /// `GET /api/broadcast/{tourSlug}/{roundSlug}/{roundId}`
    pub async fn round(
        &self,
        tour_slug: &str,
        round_slug: &str,
        round_id: &str,
    ) -> Result<LichessBroadcastRoundView> {
        let path = format!("/api/broadcast/{tour_slug}/{round_slug}/{round_id}");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "LichessBroadcastRoundView").await
    }

    /// Exports a round as PGN. `GET /api/broadcast/round/{roundId}.pgn`
    pub async fn round_pgn(&self, round_id: &str) -> Result<String> {
        let path = format!("/api/broadcast/round/{round_id}.pgn");
        http::text(self.client.request(Method::GET, Host::Default, &path)).await
    }

    /// Exports all rounds of a tournament as PGN.
    /// `GET /api/broadcast/{broadcastTournamentId}.pgn`
    pub async fn all_rounds_pgn(&self, tournament_id: &str) -> Result<String> {
        let path = format!("/api/broadcast/{tournament_id}.pgn");
        http::text(self.client.request(Method::GET, Host::Default, &path)).await
    }

    /// Streams a round's PGN as games are updated (text; stays open while the
    /// round is live). `GET /api/stream/broadcast/round/{roundId}.pgn`
    pub async fn stream_round_pgn(&self, round_id: &str) -> Result<String> {
        let path = format!("/api/stream/broadcast/round/{round_id}.pgn");
        http::text(self.client.request(Method::GET, Host::Default, &path)).await
    }

    /// Pushes PGN games to a round.
    /// `POST /api/broadcast/round/{roundId}/push`
    pub async fn push_pgn(&self, round_id: &str, pgn: &str) -> Result<LichessBroadcastPushResult> {
        let path = format!("/api/broadcast/round/{round_id}/push");
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .header(CONTENT_TYPE, "text/plain")
            .body(pgn.to_owned());
        http::json(request, "LichessBroadcastPushResult").await
    }

    /// Resets a round, removing all its games.
    /// `POST /api/broadcast/round/{roundId}/reset`
    pub async fn reset_round(&self, round_id: &str) -> Result<()> {
        let path = format!("/api/broadcast/round/{round_id}/reset");
        http::ok(self.client.request(Method::POST, Host::Default, &path)).await
    }

    /// Lists the players of a broadcast. `GET /broadcast/{id}/players`
    pub async fn players(&self, tournament_id: &str) -> Result<Vec<LichessBroadcastPlayerEntry>> {
        let path = format!("/broadcast/{tournament_id}/players");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "Vec<LichessBroadcastPlayerEntry>").await
    }

    /// Gets a single player of a broadcast.
    /// `GET /broadcast/{id}/players/{playerId}`
    pub async fn player(
        &self,
        tournament_id: &str,
        player_id: &str,
    ) -> Result<LichessBroadcastPlayerEntry> {
        let path = format!("/broadcast/{tournament_id}/players/{player_id}");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "LichessBroadcastPlayerEntry").await
    }

    /// Gets the team leaderboard of a broadcast.
    /// `GET /broadcast/{id}/teams/standings`
    pub async fn team_standings(
        &self,
        tournament_id: &str,
    ) -> Result<Vec<LichessBroadcastPlayerEntry>> {
        let path = format!("/broadcast/{tournament_id}/teams/standings");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "broadcast team standings").await
    }

    /// Starts building a new broadcast tournament. `POST /broadcast/new`
    #[must_use]
    pub fn create_tour(&self, name: &'a str) -> TourRequest<'a> {
        TourRequest::new(self.client, None, name)
    }

    /// Starts editing a broadcast tournament. `POST /broadcast/{id}/edit`
    #[must_use]
    pub fn update_tour(&self, tournament_id: &'a str, name: &'a str) -> TourRequest<'a> {
        TourRequest::new(self.client, Some(tournament_id), name)
    }

    /// Starts creating a round under a tournament. `POST /broadcast/{id}/new`
    #[must_use]
    pub fn create_round(&self, tournament_id: &'a str, name: &'a str) -> RoundRequest<'a> {
        RoundRequest::new(self.client, tournament_id, false, name)
    }

    /// Starts editing a round. `POST /broadcast/round/{roundId}/edit`
    #[must_use]
    pub fn update_round(&self, round_id: &'a str, name: &'a str) -> RoundRequest<'a> {
        RoundRequest::new(self.client, round_id, true, name)
    }
}

impl LichessClient {
    /// Broadcasts API: tournaments, rounds, players, and PGN.
    #[must_use]
    pub fn broadcasts(&self) -> BroadcastsApi<'_> {
        BroadcastsApi::new(self)
    }
}
