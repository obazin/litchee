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

pub use export::{GameExportRequest, UserGamesRequest};
pub use model::{
    LichessGame, LichessGameArenaTour, LichessGameChatMessage, LichessGameClock,
    LichessGameDivision, LichessGameMoveAnalysis, LichessGameMoveUpdate, LichessGameOpening,
    LichessGamePlayer, LichessGamePlayers, LichessGameStatusName, LichessGameSwissTour,
    LichessImportedGame, LichessMoveJudgment, LichessNowPlaying, LichessNowPlayingGame,
    LichessNowPlayingOpponent, LichessPlayerAnalysis,
};

/// The `application/x-ndjson` content type.
const NDJSON: &str = "application/x-ndjson";

/// The `application/x-chess-pgn` content type.
const PGN: &str = "application/x-chess-pgn";

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
        let path = format!("/api/user/{}/current-game", http::segment(username));
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

    /// Gets the spectator chat of a game. `GET /api/game/{gameId}/chat`
    pub async fn chat(&self, game_id: &str) -> Result<Vec<LichessGameChatMessage>> {
        let path = format!("/api/game/{}/chat", http::segment(game_id));
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "Vec<LichessGameChatMessage>").await
    }
}

impl LichessClient {
    /// Games API: export and import games.
    #[must_use]
    pub fn games(&self) -> GamesApi<'_> {
        GamesApi::new(self)
    }
}
