//! The Bot API: play games as a bot account.
//!
//! Reached through [`LichessClient::bot`]. Reuses the Board API's game-event
//! types ([`LichessBoardEvent`]).

use futures_util::stream::BoxStream;
use reqwest::Method;

use crate::api::gameplay::board::{
    ChatForm, LichessBoardEvent, LichessChatRoom, LichessIncomingEvent, yes_no,
};
use crate::api::gameplay::games::LichessGameChatMessage;
use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;
use crate::model::LichessUser;

/// Accessor for the Bot API.
#[derive(Debug)]
pub struct BotApi<'a> {
    client: &'a LichessClient,
}

impl<'a> BotApi<'a> {
    /// Binds the accessor to a client.
    pub(crate) fn new(client: &'a LichessClient) -> Self {
        Self { client }
    }

    /// Upgrades the authenticated account to a bot account (irreversible, and
    /// only for accounts with no played games).
    ///
    /// `POST /api/bot/account/upgrade`
    pub async fn upgrade_to_bot(&self) -> Result<()> {
        let request = self
            .client
            .request(Method::POST, Host::Default, "/api/bot/account/upgrade");
        http::ok(request).await
    }

    /// Streams the currently-online bots.
    ///
    /// `GET /api/bot/online`
    pub async fn online(&self) -> Result<BoxStream<'static, Result<LichessUser>>> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/bot/online");
        http::stream(request).await
    }

    /// Streams the state of a bot game.
    ///
    /// `GET /api/bot/game/stream/{gameId}`
    pub async fn stream_game(
        &self,
        game_id: &str,
    ) -> Result<BoxStream<'static, Result<LichessBoardEvent>>> {
        let path = format!("/api/bot/game/stream/{game_id}");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::stream(request).await
    }

    /// Makes a move (optionally offering or agreeing to a draw).
    ///
    /// `POST /api/bot/game/{gameId}/move/{move}`
    pub async fn make_move(
        &self,
        game_id: &str,
        chess_move: &str,
        offering_draw: bool,
    ) -> Result<()> {
        let path = format!("/api/bot/game/{game_id}/move/{chess_move}");
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .query(&[("offeringDraw", offering_draw)]);
        http::ok(request).await
    }

    /// Aborts the game. `POST /api/bot/game/{gameId}/abort`
    pub async fn abort(&self, game_id: &str) -> Result<()> {
        self.post_action(game_id, "abort").await
    }

    /// Resigns the game. `POST /api/bot/game/{gameId}/resign`
    pub async fn resign(&self, game_id: &str) -> Result<()> {
        self.post_action(game_id, "resign").await
    }

    /// Claims victory when the opponent has left.
    /// `POST /api/bot/game/{gameId}/claim-victory`
    pub async fn claim_victory(&self, game_id: &str) -> Result<()> {
        self.post_action(game_id, "claim-victory").await
    }

    /// Claims a draw when allowed. `POST /api/bot/game/{gameId}/claim-draw`
    pub async fn claim_draw(&self, game_id: &str) -> Result<()> {
        self.post_action(game_id, "claim-draw").await
    }

    /// Accepts or declines a draw offer.
    /// `POST /api/bot/game/{gameId}/draw/{accept}`
    pub async fn handle_draw(&self, game_id: &str, accept: bool) -> Result<()> {
        let path = format!("/api/bot/game/{game_id}/draw/{}", yes_no(accept));
        http::ok(self.client.request(Method::POST, Host::Default, &path)).await
    }

    /// Accepts or declines a takeback proposal.
    /// `POST /api/bot/game/{gameId}/takeback/{accept}`
    pub async fn handle_takeback(&self, game_id: &str, accept: bool) -> Result<()> {
        let path = format!("/api/bot/game/{game_id}/takeback/{}", yes_no(accept));
        http::ok(self.client.request(Method::POST, Host::Default, &path)).await
    }

    /// Posts a message to the game chat.
    /// `POST /api/bot/game/{gameId}/chat`
    pub async fn write_chat(&self, game_id: &str, room: LichessChatRoom, text: &str) -> Result<()> {
        let path = format!("/api/bot/game/{game_id}/chat");
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .form(&ChatForm { room, text });
        http::ok(request).await
    }

    /// Streams the incoming events for the authenticated bot (game starts and
    /// finishes, challenges). `GET /api/stream/event`
    pub async fn stream_events(&self) -> Result<BoxStream<'static, Result<LichessIncomingEvent>>> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/stream/event");
        http::stream(request).await
    }

    /// Reads the player chat of a bot game. `GET /api/bot/game/{gameId}/chat`
    pub async fn read_chat(&self, game_id: &str) -> Result<Vec<LichessGameChatMessage>> {
        let path = format!("/api/bot/game/{game_id}/chat");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "Vec<LichessGameChatMessage>").await
    }

    /// Issues a no-argument `POST` action on a bot game.
    async fn post_action(&self, game_id: &str, action: &str) -> Result<()> {
        let path = format!("/api/bot/game/{game_id}/{action}");
        http::ok(self.client.request(Method::POST, Host::Default, &path)).await
    }
}

impl LichessClient {
    /// Bot API: play games as a bot account.
    #[must_use]
    pub fn bot(&self) -> BotApi<'_> {
        BotApi::new(self)
    }
}
