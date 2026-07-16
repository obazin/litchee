//! The Games API: export games (JSON, NDJSON, or PGN) and import PGN.
//!
//! Reached through [`LichessClient::games`].

use reqwest::Method;

use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;

mod export;
mod model;
mod stream;

pub use export::{
    CurrentGameRequest, ExportBookmarksRequest, ExportByIdsRequest, GameExportRequest, GameSort,
    UserGamesRequest,
};
pub use model::{
    LichessGame, LichessGameArenaTour, LichessGameChatMessage, LichessGameClock,
    LichessGameDivision, LichessGameMoveAnalysis, LichessGameMoveUpdate, LichessGameOpening,
    LichessGamePlayer, LichessGamePlayers, LichessGameStatusName, LichessGameSwissTour,
    LichessImportedGame, LichessMoveJudgment, LichessNowPlaying, LichessNowPlayingGame,
    LichessNowPlayingOpponent, LichessPlayerAnalysis,
};

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

    /// Starts an export of the game a user is currently playing, if any.
    ///
    /// Finish with [`json`](CurrentGameRequest::json) or
    /// [`pgn`](CurrentGameRequest::pgn). `GET /api/user/{username}/current-game`
    #[must_use]
    pub fn current_game(&self, username: &'a str) -> CurrentGameRequest<'a> {
        CurrentGameRequest::new(self.client, username)
    }

    /// Starts an export of several games by id. `POST /api/games/export/_ids`
    ///
    /// Finish with [`stream`](ExportByIdsRequest::stream) or
    /// [`pgn`](ExportByIdsRequest::pgn).
    #[must_use]
    pub fn export_by_ids(&self, ids: &[&str]) -> ExportByIdsRequest<'a> {
        ExportByIdsRequest::new(self.client, ids)
    }

    /// Starts an export of the authenticated user's bookmarked games.
    ///
    /// Finish with [`stream`](ExportBookmarksRequest::stream) or
    /// [`pgn`](ExportBookmarksRequest::pgn). `GET /api/games/export/bookmarks`
    #[must_use]
    pub fn export_bookmarks(&self) -> ExportBookmarksRequest<'a> {
        ExportBookmarksRequest::new(self.client)
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
    /// `nb` limits the number of games returned. `GET /api/account/playing`
    pub async fn now_playing(&self, nb: Option<u32>) -> Result<LichessNowPlaying> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/account/playing")
            .query(&[("nb", nb)]);
        http::json(request, "LichessNowPlaying").await
    }

    /// Gets the spectator chat of a game. `GET /api/game/{gameId}/chat`
    pub async fn chat(&self, game_id: &str) -> Result<Vec<LichessGameChatMessage>> {
        let path = format!("/api/game/{}/chat", http::segment(game_id));
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "Vec<LichessGameChatMessage>").await
    }

    /// Bookmarks a game for the authenticated user. `POST /bookmark/{gameId}`
    ///
    /// With `set` as `None` the bookmark is toggled (added if absent, removed if
    /// present); `Some(true)` adds it and `Some(false)` removes it, making the
    /// call idempotent. Requires the `preference:write` scope.
    pub async fn bookmark(&self, game_id: &str, set: Option<bool>) -> Result<()> {
        let path = format!("/bookmark/{}", http::segment(game_id));
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .query(&[("v", set)]);
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
