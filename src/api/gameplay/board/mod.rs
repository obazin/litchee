//! The Board API: play games with a physical or third-party board.
//!
//! Reached through [`LichessClient::board`].

use futures_util::stream::BoxStream;
use reqwest::Method;
use serde::Serialize;
use serde_json::Value;

use crate::api::gameplay::games::LichessGameChatMessage;
use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;
use crate::model::LichessVariantKey;

mod events;

pub use events::{
    LichessBoardEvent, LichessChatLine, LichessChatRoom, LichessGameEventClock,
    LichessGameEventInfo, LichessGameEventPlayer, LichessGameExpiration, LichessGameFull,
    LichessGameState, LichessIncomingEvent, LichessOpponentGone, LichessVariantInfo,
};

/// Form body for posting a chat message. Shared with the Bot API.
#[derive(Debug, Serialize)]
pub(crate) struct ChatForm<'a> {
    pub(crate) room: LichessChatRoom,
    pub(crate) text: &'a str,
}

/// Accessor for the Board API.
#[derive(Debug)]
pub struct BoardApi<'a> {
    client: &'a LichessClient,
}

impl<'a> BoardApi<'a> {
    /// Binds the accessor to a client.
    pub(crate) fn new(client: &'a LichessClient) -> Self {
        Self { client }
    }

    /// Streams the state of a board game.
    ///
    /// `GET /api/board/game/stream/{gameId}`
    pub async fn stream_game(
        &self,
        game_id: &str,
    ) -> Result<BoxStream<'static, Result<LichessBoardEvent>>> {
        let path = format!("/api/board/game/stream/{game_id}");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::stream(request).await
    }

    /// Makes a move (optionally offering or agreeing to a draw).
    ///
    /// `POST /api/board/game/{gameId}/move/{move}`
    pub async fn make_move(
        &self,
        game_id: &str,
        chess_move: &str,
        offering_draw: bool,
    ) -> Result<()> {
        let path = format!("/api/board/game/{game_id}/move/{chess_move}");
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .query(&[("offeringDraw", offering_draw)]);
        http::ok(request).await
    }

    /// Aborts the game. `POST /api/board/game/{gameId}/abort`
    pub async fn abort(&self, game_id: &str) -> Result<()> {
        self.post_action(game_id, "abort").await
    }

    /// Resigns the game. `POST /api/board/game/{gameId}/resign`
    pub async fn resign(&self, game_id: &str) -> Result<()> {
        self.post_action(game_id, "resign").await
    }

    /// Claims victory when the opponent has left.
    /// `POST /api/board/game/{gameId}/claim-victory`
    pub async fn claim_victory(&self, game_id: &str) -> Result<()> {
        self.post_action(game_id, "claim-victory").await
    }

    /// Claims a draw when allowed. `POST /api/board/game/{gameId}/claim-draw`
    pub async fn claim_draw(&self, game_id: &str) -> Result<()> {
        self.post_action(game_id, "claim-draw").await
    }

    /// Goes berserk on an arena game. `POST /api/board/game/{gameId}/berserk`
    pub async fn berserk(&self, game_id: &str) -> Result<()> {
        self.post_action(game_id, "berserk").await
    }

    /// Accepts or declines a draw offer.
    /// `POST /api/board/game/{gameId}/draw/{accept}`
    pub async fn handle_draw(&self, game_id: &str, accept: bool) -> Result<()> {
        let path = format!("/api/board/game/{game_id}/draw/{}", yes_no(accept));
        http::ok(self.client.request(Method::POST, Host::Default, &path)).await
    }

    /// Accepts or declines a takeback proposal.
    /// `POST /api/board/game/{gameId}/takeback/{accept}`
    pub async fn handle_takeback(&self, game_id: &str, accept: bool) -> Result<()> {
        let path = format!("/api/board/game/{game_id}/takeback/{}", yes_no(accept));
        http::ok(self.client.request(Method::POST, Host::Default, &path)).await
    }

    /// Posts a message to the game chat.
    /// `POST /api/board/game/{gameId}/chat`
    pub async fn write_chat(&self, game_id: &str, room: LichessChatRoom, text: &str) -> Result<()> {
        let path = format!("/api/board/game/{game_id}/chat");
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .form(&ChatForm { room, text });
        http::ok(request).await
    }

    /// Streams the incoming events for the authenticated user (game starts and
    /// finishes, challenges). `GET /api/stream/event`
    ///
    /// Only one such stream can be active per token at a time.
    pub async fn stream_events(&self) -> Result<BoxStream<'static, Result<LichessIncomingEvent>>> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/stream/event");
        http::stream(request).await
    }

    /// Reads the player chat of a board game.
    /// `GET /api/board/game/{gameId}/chat`
    pub async fn read_chat(&self, game_id: &str) -> Result<Vec<LichessGameChatMessage>> {
        let path = format!("/api/board/game/{game_id}/chat");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "Vec<LichessGameChatMessage>").await
    }

    /// Creates a public seek to start a game with a random opponent.
    ///
    /// Hold the returned stream open to keep the seek active; dropping it
    /// cancels the seek. `POST /api/board/seek`
    #[must_use]
    pub fn seek(&self) -> SeekRequest<'_> {
        SeekRequest::new(self.client)
    }

    /// Issues a no-argument `POST` action on a board game.
    async fn post_action(&self, game_id: &str, action: &str) -> Result<()> {
        let path = format!("/api/board/game/{game_id}/{action}");
        http::ok(self.client.request(Method::POST, Host::Default, &path)).await
    }
}

/// Form body for a seek.
#[derive(Debug, Default, Serialize)]
struct SeekForm<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    rated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    time: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    increment: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    days: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    variant: Option<LichessVariantKey>,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<&'a str>,
    #[serde(rename = "ratingRange", skip_serializing_if = "Option::is_none")]
    rating_range: Option<&'a str>,
}

/// Builder for a [`BoardApi::seek`] request.
#[derive(Debug)]
pub struct SeekRequest<'a> {
    client: &'a LichessClient,
    form: SeekForm<'a>,
}

impl<'a> SeekRequest<'a> {
    /// Creates the request builder.
    fn new(client: &'a LichessClient) -> Self {
        Self {
            client,
            form: SeekForm::default(),
        }
    }

    /// Sets whether the game is rated.
    #[must_use]
    pub fn rated(mut self, rated: bool) -> Self {
        self.form.rated = Some(rated);
        self
    }

    /// Sets a real-time clock (initial minutes + increment seconds).
    #[must_use]
    pub fn clock(mut self, time_minutes: f32, increment_secs: u32) -> Self {
        self.form.time = Some(time_minutes);
        self.form.increment = Some(increment_secs);
        self
    }

    /// Sets days per turn for a correspondence seek.
    #[must_use]
    pub fn days(mut self, days: u32) -> Self {
        self.form.days = Some(days);
        self
    }

    /// Sets the variant.
    #[must_use]
    pub fn variant(mut self, variant: LichessVariantKey) -> Self {
        self.form.variant = Some(variant);
        self
    }

    /// Sets the color to play (`"white"`, `"black"`, or `"random"`).
    #[must_use]
    pub fn color(mut self, color: &'a str) -> Self {
        self.form.color = Some(color);
        self
    }

    /// Sends the seek, returning a stream to hold open.
    pub async fn send(self) -> Result<BoxStream<'static, Result<Value>>> {
        let request = self
            .client
            .request(Method::POST, Host::Default, "/api/board/seek")
            .form(&self.form);
        http::stream(request).await
    }
}

impl LichessClient {
    /// Board API: play games with a physical or third-party board.
    #[must_use]
    pub fn board(&self) -> BoardApi<'_> {
        BoardApi::new(self)
    }
}

/// Renders a yes/no path segment for accept-style endpoints. Shared with the
/// Bot API.
pub(crate) fn yes_no(accept: bool) -> &'static str {
    if accept { "yes" } else { "no" }
}
