//! The Broadcasts API: official broadcasts, rounds, players, and PGN.
//!
//! Reached through [`LichessClient::broadcasts`].

use futures_util::stream::BoxStream;
use reqwest::Method;

use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;
use crate::model::PgnExportOptions;

mod model;
mod round;
mod tour;

pub use model::{
    LichessBroadcast, LichessBroadcastMyRound, LichessBroadcastPlayerEntry,
    LichessBroadcastPushResult, LichessBroadcastRoundInfo, LichessBroadcastRoundView,
    LichessBroadcastSearchPage, LichessBroadcastTop, LichessBroadcastTour,
};
pub use round::{BroadcastCustomPoints, BroadcastCustomScoring, RoundRequest};
pub use tour::{BroadcastGrouping, BroadcastTourInfo, TourRequest};

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
    ///
    /// `nb` limits the count; `html` embeds rendered descriptions; `live`
    /// restricts to broadcasts with an ongoing round.
    pub async fn official(
        &self,
        nb: Option<u32>,
        html: Option<bool>,
        live: Option<bool>,
    ) -> Result<BoxStream<'static, Result<LichessBroadcast>>> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/broadcast")
            .query(&[("nb", nb)])
            .query(&[("html", html), ("live", live)]);
        http::stream(request, self.client.max_line_bytes()).await
    }

    /// Gets the top broadcasts (active, upcoming, past). `GET /api/broadcast/top`
    ///
    /// `page` selects a page; `html` embeds rendered descriptions.
    pub async fn top(&self, page: Option<u32>, html: Option<bool>) -> Result<LichessBroadcastTop> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/broadcast/top")
            .query(&[("page", page)])
            .query(&[("html", html)]);
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
    ///
    /// `page` selects a page; `html` embeds rendered descriptions.
    ///
    /// Requires an OAuth token with (at least) the
    /// [`study:read`](crate::api::auth::oauth::Scope::StudyRead) scope.
    pub async fn by_user(
        &self,
        username: &str,
        page: Option<u32>,
        html: Option<bool>,
    ) -> Result<BoxStream<'static, Result<LichessBroadcast>>> {
        let path = format!("/api/broadcast/by/{}", http::segment(username));
        let request = self
            .client
            .request(Method::GET, Host::Default, &path)
            .query(&[("page", page)])
            .query(&[("html", html)]);
        http::stream(request, self.client.max_line_bytes()).await
    }

    /// Streams the authenticated user's broadcast rounds.
    ///
    /// `nb` limits the number of rounds. `GET /api/broadcast/my-rounds`
    ///
    /// Requires an OAuth token with (at least) the
    /// [`study:read`](crate::api::auth::oauth::Scope::StudyRead) scope.
    pub async fn my_rounds(
        &self,
        nb: Option<u32>,
    ) -> Result<BoxStream<'static, Result<LichessBroadcastMyRound>>> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/broadcast/my-rounds")
            .query(&[("nb", nb)]);
        http::stream(request, self.client.max_line_bytes()).await
    }

    /// Gets a broadcast tournament with its rounds.
    /// `GET /api/broadcast/{broadcastTournamentId}`
    ///
    /// Requires an OAuth token with (at least) the
    /// [`study:read`](crate::api::auth::oauth::Scope::StudyRead) scope.
    pub async fn get_tournament(&self, tournament_id: &str) -> Result<LichessBroadcast> {
        let path = format!("/api/broadcast/{}", http::segment(tournament_id));
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "LichessBroadcast").await
    }

    /// Gets a round with its games.
    /// `GET /api/broadcast/{tourSlug}/{roundSlug}/{roundId}`
    ///
    /// Requires an OAuth token with (at least) the
    /// [`study:read`](crate::api::auth::oauth::Scope::StudyRead) scope.
    pub async fn round(
        &self,
        tour_slug: &str,
        round_slug: &str,
        round_id: &str,
    ) -> Result<LichessBroadcastRoundView> {
        let path = format!(
            "/api/broadcast/{}/{}/{}",
            http::segment(tour_slug),
            http::segment(round_slug),
            http::segment(round_id)
        );
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "LichessBroadcastRoundView").await
    }

    /// Exports a round as PGN. `GET /api/broadcast/round/{roundId}.pgn`
    ///
    /// Requires an OAuth token with (at least) the
    /// [`study:read`](crate::api::auth::oauth::Scope::StudyRead) scope.
    pub async fn round_pgn(&self, round_id: &str, options: &PgnExportOptions) -> Result<String> {
        let path = format!("/api/broadcast/round/{}.pgn", http::segment(round_id));
        let request = self
            .client
            .request(Method::GET, Host::Default, &path)
            .query(options);
        http::text(request).await
    }

    /// Exports all rounds of a tournament as PGN.
    /// `GET /api/broadcast/{broadcastTournamentId}.pgn`
    ///
    /// For real-time updates about an ongoing tournament, prefer the round PGN
    /// stream ([`Self::stream_round_pgn`]), tournament PGN stream
    /// ([`Self::stream_tour_pgn`]), or group PGN stream
    /// ([`Self::stream_group_pgn`]) instead.
    pub async fn all_rounds_pgn(
        &self,
        tournament_id: &str,
        options: &PgnExportOptions,
    ) -> Result<String> {
        let path = format!("/api/broadcast/{}.pgn", http::segment(tournament_id));
        let request = self
            .client
            .request(Method::GET, Host::Default, &path)
            .query(options);
        http::text(request).await
    }

    /// Streams a round's PGN as games are updated (text; stays open while the
    /// round is live). `GET /api/stream/broadcast/round/{roundId}.pgn`
    ///
    /// Requires an OAuth token with (at least) the
    /// [`study:read`](crate::api::auth::oauth::Scope::StudyRead) scope.
    pub async fn stream_round_pgn(
        &self,
        round_id: &str,
        options: &PgnExportOptions,
    ) -> Result<String> {
        let path = format!(
            "/api/stream/broadcast/round/{}.pgn",
            http::segment(round_id)
        );
        let request = self
            .client
            .request(Method::GET, Host::Default, &path)
            .query(options);
        http::text(request).await
    }

    /// Streams the PGN of all ongoing rounds of a broadcast group as games are
    /// updated (text; stays open while rounds are live).
    /// `GET /api/stream/broadcast/group/{broadcastGroupId}.pgn`
    ///
    /// Requires an OAuth token with (at least) the
    /// [`study:read`](crate::api::auth::oauth::Scope::StudyRead) scope.
    pub async fn stream_group_pgn(
        &self,
        group_id: &str,
        options: &PgnExportOptions,
    ) -> Result<String> {
        let path = format!(
            "/api/stream/broadcast/group/{}.pgn",
            http::segment(group_id)
        );
        let request = self
            .client
            .request(Method::GET, Host::Default, &path)
            .query(options);
        http::text(request).await
    }

    /// Streams the PGN of all ongoing rounds of a broadcast tournament as games
    /// are updated (text; stays open while rounds are live).
    /// `GET /api/stream/broadcast/tour/{broadcastTourId}.pgn`
    ///
    /// Requires an OAuth token with (at least) the
    /// [`study:read`](crate::api::auth::oauth::Scope::StudyRead) scope.
    pub async fn stream_tour_pgn(
        &self,
        tour_id: &str,
        options: &PgnExportOptions,
    ) -> Result<String> {
        let path = format!("/api/stream/broadcast/tour/{}.pgn", http::segment(tour_id));
        let request = self
            .client
            .request(Method::GET, Host::Default, &path)
            .query(options);
        http::text(request).await
    }

    /// Pushes PGN games to a round.
    /// `POST /api/broadcast/round/{roundId}/push`
    pub async fn push_pgn(&self, round_id: &str, pgn: &str) -> Result<LichessBroadcastPushResult> {
        let path = format!("/api/broadcast/round/{}/push", http::segment(round_id));
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .text_body(pgn.to_owned());
        http::json(request, "LichessBroadcastPushResult").await
    }

    /// Resets a round, removing all its games.
    /// `POST /api/broadcast/round/{roundId}/reset`
    pub async fn reset_round(&self, round_id: &str) -> Result<()> {
        let path = format!("/api/broadcast/round/{}/reset", http::segment(round_id));
        http::ok(self.client.request(Method::POST, Host::Default, &path)).await
    }

    /// Lists the players of a broadcast. `GET /broadcast/{id}/players`
    pub async fn players(&self, tournament_id: &str) -> Result<Vec<LichessBroadcastPlayerEntry>> {
        let path = format!("/broadcast/{}/players", http::segment(tournament_id));
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
        let path = format!(
            "/broadcast/{}/players/{}",
            http::segment(tournament_id),
            http::segment(player_id)
        );
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "LichessBroadcastPlayerEntry").await
    }

    /// Gets the team leaderboard of a broadcast.
    /// `GET /broadcast/{id}/teams/standings`
    pub async fn team_standings(
        &self,
        tournament_id: &str,
    ) -> Result<Vec<LichessBroadcastPlayerEntry>> {
        let path = format!(
            "/broadcast/{}/teams/standings",
            http::segment(tournament_id)
        );
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
    ///
    /// The edit endpoint **replaces** the round: any field left unset is blanked
    /// (dropping the existing sync source, start time, etc.). Call
    /// [`RoundRequest::patch`] with `true` to instead update only the fields you
    /// set and leave the rest untouched.
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
