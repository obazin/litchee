//! Builders for the game-export endpoints (single game, a user's games,
//! by-ids, bookmarks, and the current game).

use futures_util::stream::BoxStream;
use reqwest::Method;
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use serde::Serialize;

use super::model::LichessGame;
use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;
use crate::http::{ACCEPT_JSON, ACCEPT_NDJSON, ACCEPT_PGN};
use crate::model::GameExportOptions;

/// Sort order for a games export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GameSort {
    /// Oldest games first.
    DateAsc,
    /// Most recent games first (the server default).
    DateDesc,
}

/// Builder for exporting a single game (`GET /game/export/{gameId}`).
#[derive(Debug)]
pub struct GameExportRequest<'a> {
    client: &'a LichessClient,
    game_id: &'a str,
    filters: SingleGameFilters,
    export: GameExportOptions,
}

/// The single-game filter that is not part of the shared export block.
#[derive(Debug, Default, Serialize)]
struct SingleGameFilters {
    #[serde(rename = "withBookmarked", skip_serializing_if = "Option::is_none")]
    with_bookmarked: Option<bool>,
}

impl<'a> GameExportRequest<'a> {
    /// Creates the request builder.
    pub(crate) fn new(client: &'a LichessClient, game_id: &'a str) -> Self {
        Self {
            client,
            game_id,
            filters: SingleGameFilters::default(),
            export: GameExportOptions::default(),
        }
    }

    /// Sets the shared export-format options (moves, clocks, evals, …).
    #[must_use]
    pub fn export(mut self, options: GameExportOptions) -> Self {
        self.export = options;
        self
    }

    /// Adds a `bookmarked` flag when the caller has bookmarked the game.
    #[must_use]
    pub fn with_bookmarked(mut self, value: bool) -> Self {
        self.filters.with_bookmarked = Some(value);
        self
    }

    /// Executes the export, returning the game as JSON.
    pub async fn json(self) -> Result<LichessGame> {
        let path = format!("/game/export/{}", http::segment(self.game_id));
        let request = self
            .client
            .request(Method::GET, Host::Default, &path)
            .header(ACCEPT, ACCEPT_JSON)
            .query(&self.filters)
            .query(&self.export);
        http::json(request, "LichessGame").await
    }

    /// Executes the export, returning the game as a PGN string.
    pub async fn pgn(self) -> Result<String> {
        let path = format!("/game/export/{}", http::segment(self.game_id));
        let request = self
            .client
            .request(Method::GET, Host::Default, &path)
            .header(ACCEPT, ACCEPT_PGN)
            .query(&self.filters)
            .query(&self.export);
        http::text(request).await
    }
}

/// Builder for exporting a user's games (`GET /api/games/user/{username}`).
#[derive(Debug)]
pub struct UserGamesRequest<'a> {
    client: &'a LichessClient,
    username: &'a str,
    query: UserGamesQuery<'a>,
    export: GameExportOptions,
}

/// Filter parameters for exporting a user's games (the export-format flags live
/// in the shared [`GameExportOptions`]).
#[derive(Debug, Default, Serialize)]
struct UserGamesQuery<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    max: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    since: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    until: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vs: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rated: Option<bool>,
    #[serde(rename = "perfType", skip_serializing_if = "Option::is_none")]
    perf_type: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    analysed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ongoing: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    finished: Option<bool>,
    #[serde(rename = "lastFen", skip_serializing_if = "Option::is_none")]
    last_fen: Option<bool>,
    #[serde(rename = "withBookmarked", skip_serializing_if = "Option::is_none")]
    with_bookmarked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sort: Option<GameSort>,
}

impl<'a> UserGamesRequest<'a> {
    /// Creates the request builder.
    pub(crate) fn new(client: &'a LichessClient, username: &'a str) -> Self {
        Self {
            client,
            username,
            query: UserGamesQuery::default(),
            export: GameExportOptions::default(),
        }
    }

    /// Limits the number of games downloaded.
    #[must_use]
    pub fn max(mut self, count: u32) -> Self {
        self.query.max = Some(count);
        self
    }

    /// Only games played since this timestamp (Unix milliseconds).
    #[must_use]
    pub fn since(mut self, timestamp: i64) -> Self {
        self.query.since = Some(timestamp);
        self
    }

    /// Only games played until this timestamp (Unix milliseconds).
    #[must_use]
    pub fn until(mut self, timestamp: i64) -> Self {
        self.query.until = Some(timestamp);
        self
    }

    /// Only games played against this opponent.
    #[must_use]
    pub fn vs(mut self, opponent: &'a str) -> Self {
        self.query.vs = Some(opponent);
        self
    }

    /// Only rated (`true`) or casual (`false`) games.
    #[must_use]
    pub fn rated(mut self, rated: bool) -> Self {
        self.query.rated = Some(rated);
        self
    }

    /// Only games in these speeds/variants (comma-separated perf types).
    #[must_use]
    pub fn perf_type(mut self, perf_type: &'a str) -> Self {
        self.query.perf_type = Some(perf_type);
        self
    }

    /// Only games played as this color (`"white"` or `"black"`).
    #[must_use]
    pub fn color(mut self, color: &'a str) -> Self {
        self.query.color = Some(color);
        self
    }

    /// Only games with (`true`) or without (`false`) computer analysis.
    #[must_use]
    pub fn analysed(mut self, analysed: bool) -> Self {
        self.query.analysed = Some(analysed);
        self
    }

    /// Only currently-ongoing games.
    #[must_use]
    pub fn ongoing(mut self, ongoing: bool) -> Self {
        self.query.ongoing = Some(ongoing);
        self
    }

    /// Include finished games (`true`); set `false` to get only ongoing games.
    #[must_use]
    pub fn finished(mut self, finished: bool) -> Self {
        self.query.finished = Some(finished);
        self
    }

    /// Include the X-FEN of each game's last position (NDJSON only).
    #[must_use]
    pub fn last_fen(mut self, value: bool) -> Self {
        self.query.last_fen = Some(value);
        self
    }

    /// Add a `bookmarked` flag on games the caller has bookmarked (NDJSON only).
    #[must_use]
    pub fn with_bookmarked(mut self, value: bool) -> Self {
        self.query.with_bookmarked = Some(value);
        self
    }

    /// Sort order (defaults to most-recent-first).
    #[must_use]
    pub fn sort(mut self, sort: GameSort) -> Self {
        self.query.sort = Some(sort);
        self
    }

    /// Sets the shared export-format options (moves, clocks, evals, …).
    #[must_use]
    pub fn export(mut self, options: GameExportOptions) -> Self {
        self.export = options;
        self
    }

    /// Executes the export, streaming games as decoded JSON values.
    pub async fn stream(self) -> Result<BoxStream<'static, Result<LichessGame>>> {
        let path = format!("/api/games/user/{}", http::segment(self.username));
        let request = self
            .client
            .request(Method::GET, Host::Default, &path)
            .header(ACCEPT, ACCEPT_NDJSON)
            .query(&self.query)
            .query(&self.export);
        http::stream(request, self.client.max_line_bytes()).await
    }

    /// Executes the export, returning all games as one PGN string.
    pub async fn pgn(self) -> Result<String> {
        let path = format!("/api/games/user/{}", http::segment(self.username));
        let request = self
            .client
            .request(Method::GET, Host::Default, &path)
            .header(ACCEPT, ACCEPT_PGN)
            .query(&self.query)
            .query(&self.export);
        http::text(request).await
    }
}

/// Builder for exporting several games by id (`POST /api/games/export/_ids`).
#[derive(Debug)]
pub struct ExportByIdsRequest<'a> {
    client: &'a LichessClient,
    ids: String,
    export: GameExportOptions,
}

impl<'a> ExportByIdsRequest<'a> {
    /// Creates the request builder from the game ids.
    pub(crate) fn new(client: &'a LichessClient, ids: &[&str]) -> Self {
        Self {
            client,
            ids: ids.join(","),
            export: GameExportOptions::default(),
        }
    }

    /// Sets the shared export-format options (moves, clocks, evals, …).
    #[must_use]
    pub fn export(mut self, options: GameExportOptions) -> Self {
        self.export = options;
        self
    }

    /// Executes the export, streaming games as decoded JSON values.
    pub async fn stream(self) -> Result<BoxStream<'static, Result<LichessGame>>> {
        let request = self
            .client
            .request(Method::POST, Host::Default, "/api/games/export/_ids")
            .header(ACCEPT, ACCEPT_NDJSON)
            .header(CONTENT_TYPE, "text/plain")
            .query(&self.export)
            .body(self.ids);
        http::stream(request, self.client.max_line_bytes()).await
    }

    /// Executes the export, returning all games as one PGN string.
    pub async fn pgn(self) -> Result<String> {
        let request = self
            .client
            .request(Method::POST, Host::Default, "/api/games/export/_ids")
            .header(ACCEPT, ACCEPT_PGN)
            .header(CONTENT_TYPE, "text/plain")
            .query(&self.export)
            .body(self.ids);
        http::text(request).await
    }
}

/// Builder for exporting bookmarked games (`GET /api/games/export/bookmarks`).
#[derive(Debug)]
pub struct ExportBookmarksRequest<'a> {
    client: &'a LichessClient,
    query: BookmarksQuery,
    export: GameExportOptions,
}

/// Filter parameters for the bookmarked-games export.
#[derive(Debug, Default, Serialize)]
struct BookmarksQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    since: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    until: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max: Option<u32>,
    #[serde(rename = "lastFen", skip_serializing_if = "Option::is_none")]
    last_fen: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sort: Option<GameSort>,
}

impl<'a> ExportBookmarksRequest<'a> {
    /// Creates the request builder.
    pub(crate) fn new(client: &'a LichessClient) -> Self {
        Self {
            client,
            query: BookmarksQuery::default(),
            export: GameExportOptions::default(),
        }
    }

    /// Only games played since this timestamp (Unix milliseconds).
    #[must_use]
    pub fn since(mut self, timestamp: i64) -> Self {
        self.query.since = Some(timestamp);
        self
    }

    /// Only games played until this timestamp (Unix milliseconds).
    #[must_use]
    pub fn until(mut self, timestamp: i64) -> Self {
        self.query.until = Some(timestamp);
        self
    }

    /// Limits the number of games downloaded.
    #[must_use]
    pub fn max(mut self, count: u32) -> Self {
        self.query.max = Some(count);
        self
    }

    /// Include the X-FEN of each game's last position.
    #[must_use]
    pub fn last_fen(mut self, value: bool) -> Self {
        self.query.last_fen = Some(value);
        self
    }

    /// Sort order (defaults to most-recent-first).
    #[must_use]
    pub fn sort(mut self, sort: GameSort) -> Self {
        self.query.sort = Some(sort);
        self
    }

    /// Sets the shared export-format options (moves, clocks, evals, …).
    #[must_use]
    pub fn export(mut self, options: GameExportOptions) -> Self {
        self.export = options;
        self
    }

    /// Executes the export, streaming games as decoded JSON values.
    pub async fn stream(self) -> Result<BoxStream<'static, Result<LichessGame>>> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/games/export/bookmarks")
            .header(ACCEPT, ACCEPT_NDJSON)
            .query(&self.query)
            .query(&self.export);
        http::stream(request, self.client.max_line_bytes()).await
    }

    /// Executes the export, returning all games as one PGN string.
    pub async fn pgn(self) -> Result<String> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/games/export/bookmarks")
            .header(ACCEPT, ACCEPT_PGN)
            .query(&self.query)
            .query(&self.export);
        http::text(request).await
    }
}

/// Builder for exporting a user's current game
/// (`GET /api/user/{username}/current-game`).
#[derive(Debug)]
pub struct CurrentGameRequest<'a> {
    client: &'a LichessClient,
    username: &'a str,
    export: GameExportOptions,
}

impl<'a> CurrentGameRequest<'a> {
    /// Creates the request builder.
    pub(crate) fn new(client: &'a LichessClient, username: &'a str) -> Self {
        Self {
            client,
            username,
            export: GameExportOptions::default(),
        }
    }

    /// Sets the shared export-format options (moves, clocks, evals, …).
    #[must_use]
    pub fn export(mut self, options: GameExportOptions) -> Self {
        self.export = options;
        self
    }

    /// Executes the export, returning the game as JSON.
    pub async fn json(self) -> Result<LichessGame> {
        let path = format!("/api/user/{}/current-game", http::segment(self.username));
        let request = self
            .client
            .request(Method::GET, Host::Default, &path)
            .header(ACCEPT, ACCEPT_JSON)
            .query(&self.export);
        http::json(request, "LichessGame").await
    }

    /// Executes the export, returning the game as a PGN string.
    pub async fn pgn(self) -> Result<String> {
        let path = format!("/api/user/{}/current-game", http::segment(self.username));
        let request = self
            .client
            .request(Method::GET, Host::Default, &path)
            .header(ACCEPT, ACCEPT_PGN)
            .query(&self.export);
        http::text(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_sort_uses_camel_case() {
        assert_eq!(
            serde_urlencoded::to_string([("sort", GameSort::DateAsc)]).unwrap(),
            "sort=dateAsc"
        );
        assert_eq!(
            serde_urlencoded::to_string([("sort", GameSort::DateDesc)]).unwrap(),
            "sort=dateDesc"
        );
    }
}
