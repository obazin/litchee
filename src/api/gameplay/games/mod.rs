//! The Games API: export games (JSON, NDJSON, or PGN) and import PGN.
//!
//! Reached through [`LichessClient::games`].

use futures_util::stream::BoxStream;
use reqwest::Method;
use reqwest::header::{ACCEPT, CONTENT_TYPE};

use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;

mod export;
mod model;

pub use export::{GameExportRequest, UserGamesRequest};
pub use model::{
    LichessGame, LichessGameChatMessage, LichessGameClock, LichessGameDivision,
    LichessGameMoveAnalysis, LichessGameMoveUpdate, LichessGameOpening, LichessGamePlayer,
    LichessGamePlayers, LichessGameStatusName, LichessImportedGame, LichessMoveJudgment,
    LichessNowPlaying, LichessNowPlayingGame, LichessNowPlayingOpponent, LichessPlayerAnalysis,
};

/// The `application/x-ndjson` content type.
const NDJSON: &str = "application/x-ndjson";

/// Accessor for the Games API.
#[derive(Debug)]
pub struct GamesApi<'a> {
    client: &'a LichessClient,
}

impl<'a> GamesApi<'a> {
    /// Binds the accessor to a client.
    pub(crate) fn new(client: &'a LichessClient) -> Self {
        Self { client }
    }

    /// Starts a single-game export. `GET /game/export/{gameId}`
    ///
    /// Finish with [`json`](GameExportRequest::json) or
    /// [`pgn`](GameExportRequest::pgn).
    #[must_use]
    pub fn export(&self, game_id: &'a str) -> GameExportRequest<'a> {
        GameExportRequest::new(self.client, game_id)
    }

    /// Starts a user's games export. `GET /api/games/user/{username}`
    ///
    /// Finish with [`stream`](UserGamesRequest::stream) or
    /// [`pgn`](UserGamesRequest::pgn).
    #[must_use]
    pub fn export_user(&self, username: &'a str) -> UserGamesRequest<'a> {
        UserGamesRequest::new(self.client, username)
    }

    /// Gets the game the user is currently playing, if any.
    ///
    /// `GET /api/user/{username}/current-game`
    pub async fn current_game(&self, username: &str) -> Result<LichessGame> {
        let path = format!("/api/user/{username}/current-game");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "LichessGame").await
    }

    /// Imports a game from its PGN.
    ///
    /// Requires authentication. `POST /api/import`
    pub async fn import_game(&self, pgn: &str) -> Result<LichessImportedGame> {
        let request = self
            .client
            .request(Method::POST, Host::Default, "/api/import")
            .form(&[("pgn", pgn)]);
        http::json(request, "LichessImportedGame").await
    }

    /// Gets the games the authenticated user is currently playing.
    ///
    /// `GET /api/account/playing`
    pub async fn now_playing(&self) -> Result<LichessNowPlaying> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/account/playing");
        http::json(request, "LichessNowPlaying").await
    }

    /// Gets the spectator chat of a game. `GET /game/{gameId}/chat`
    pub async fn chat(&self, game_id: &str) -> Result<Vec<LichessGameChatMessage>> {
        let path = format!("/game/{game_id}/chat");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "Vec<LichessGameChatMessage>").await
    }

    /// Exports several games by id (NDJSON). `POST /api/games/export/_ids`
    pub async fn export_by_ids(
        &self,
        ids: &[&str],
    ) -> Result<BoxStream<'static, Result<LichessGame>>> {
        let request = self
            .client
            .request(Method::POST, Host::Default, "/api/games/export/_ids")
            .header(ACCEPT, NDJSON)
            .header(CONTENT_TYPE, "text/plain")
            .body(ids.join(","));
        http::stream(request).await
    }

    /// Streams the authenticated user's bookmarked games (NDJSON).
    /// `GET /api/games/export/bookmarks`
    pub async fn export_bookmarks(&self) -> Result<BoxStream<'static, Result<LichessGame>>> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/games/export/bookmarks")
            .header(ACCEPT, NDJSON);
        http::stream(request).await
    }

    /// Streams the authenticated user's imported games (NDJSON).
    /// `GET /api/games/export/imports`
    pub async fn export_imports(&self) -> Result<BoxStream<'static, Result<LichessGame>>> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/games/export/imports")
            .header(ACCEPT, NDJSON);
        http::stream(request).await
    }

    /// Streams a game's moves as they are played. `GET /api/stream/game/{id}`
    pub async fn stream_moves(
        &self,
        game_id: &str,
    ) -> Result<BoxStream<'static, Result<LichessGameMoveUpdate>>> {
        let path = format!("/api/stream/game/{game_id}");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::stream(request).await
    }

    /// Streams games played by the given users as they start/finish (NDJSON).
    /// `POST /api/stream/games-by-users`
    pub async fn stream_by_users(
        &self,
        usernames: &[&str],
    ) -> Result<BoxStream<'static, Result<LichessGame>>> {
        let request = self
            .client
            .request(Method::POST, Host::Default, "/api/stream/games-by-users")
            .header(CONTENT_TYPE, "text/plain")
            .body(usernames.join(","));
        http::stream(request).await
    }

    /// Streams a custom set of games by id (NDJSON).
    /// `POST /api/stream/games/{streamId}`
    pub async fn stream_by_ids(
        &self,
        stream_id: &str,
        ids: &[&str],
    ) -> Result<BoxStream<'static, Result<LichessGame>>> {
        let path = format!("/api/stream/games/{stream_id}");
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .header(CONTENT_TYPE, "text/plain")
            .body(ids.join(","));
        http::stream(request).await
    }

    /// Adds game ids to an existing game stream.
    /// `POST /api/stream/games/{streamId}/add`
    pub async fn add_to_stream(&self, stream_id: &str, ids: &[&str]) -> Result<()> {
        let path = format!("/api/stream/games/{stream_id}/add");
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .header(CONTENT_TYPE, "text/plain")
            .body(ids.join(","));
        http::ok(request).await
    }
}

impl LichessClient {
    /// Games API: export and import games.
    #[must_use]
    pub fn games(&self) -> GamesApi<'_> {
        GamesApi::new(self)
    }
}
