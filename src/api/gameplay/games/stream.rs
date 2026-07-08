//! Streaming and bulk export methods on [`GamesApi`] (NDJSON, no query builder).

use futures_util::stream::BoxStream;
use reqwest::Method;
use reqwest::header::{ACCEPT, CONTENT_TYPE};

use super::model::{LichessGame, LichessGameMoveUpdate};
use super::{GamesApi, NDJSON};
use crate::config::Host;
use crate::error::Result;
use crate::http;

impl GamesApi<'_> {
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
        http::stream(request, self.client.max_line_bytes()).await
    }

    /// Streams the authenticated user's bookmarked games (NDJSON).
    /// `GET /api/games/export/bookmarks`
    pub async fn export_bookmarks(&self) -> Result<BoxStream<'static, Result<LichessGame>>> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/games/export/bookmarks")
            .header(ACCEPT, NDJSON);
        http::stream(request, self.client.max_line_bytes()).await
    }

    /// Streams the authenticated user's imported games (NDJSON).
    /// `GET /api/games/export/imports`
    pub async fn export_imports(&self) -> Result<BoxStream<'static, Result<LichessGame>>> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/games/export/imports")
            .header(ACCEPT, NDJSON);
        http::stream(request, self.client.max_line_bytes()).await
    }

    /// Streams a game's moves as they are played. `GET /api/stream/game/{id}`
    pub async fn stream_moves(
        &self,
        game_id: &str,
    ) -> Result<BoxStream<'static, Result<LichessGameMoveUpdate>>> {
        let path = format!("/api/stream/game/{}", http::segment(game_id));
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::stream(request, self.client.max_line_bytes()).await
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
        http::stream(request, self.client.max_line_bytes()).await
    }

    /// Streams a custom set of games by id (NDJSON).
    /// `POST /api/stream/games/{streamId}`
    pub async fn stream_by_ids(
        &self,
        stream_id: &str,
        ids: &[&str],
    ) -> Result<BoxStream<'static, Result<LichessGame>>> {
        let path = format!("/api/stream/games/{}", http::segment(stream_id));
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .header(CONTENT_TYPE, "text/plain")
            .body(ids.join(","));
        http::stream(request, self.client.max_line_bytes()).await
    }

    /// Adds game ids to an existing game stream.
    /// `POST /api/stream/games/{streamId}/add`
    pub async fn add_to_stream(&self, stream_id: &str, ids: &[&str]) -> Result<()> {
        let path = format!("/api/stream/games/{}/add", http::segment(stream_id));
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .header(CONTENT_TYPE, "text/plain")
            .body(ids.join(","));
        http::ok(request).await
    }
}
