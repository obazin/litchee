//! The Puzzles API: daily puzzle, lookups, the next puzzle, and activity.
//!
//! Reached through [`LichessClient::puzzles`].

use futures_util::stream::BoxStream;
use reqwest::Method;
use serde::Serialize;

use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;

mod model;

pub use model::{
    LichessPuzzle, LichessPuzzleActivity, LichessPuzzleAndGame, LichessPuzzleBatch,
    LichessPuzzleDashboard, LichessPuzzleGame, LichessPuzzlePerf, LichessPuzzlePlayer,
    LichessPuzzleRacer, LichessPuzzleReplay, LichessPuzzleSolution, LichessStormDashboard,
};

/// Accessor for the Puzzles API.
#[derive(Debug)]
pub struct PuzzlesApi<'a> {
    client: &'a LichessClient,
}

/// Query parameters for the next puzzle.
#[derive(Debug, Default, Serialize)]
struct NextQuery<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    angle: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    difficulty: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<&'a str>,
}

/// Query parameters for puzzle activity.
#[derive(Debug, Default, Serialize)]
struct ActivityQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    max: Option<u32>,
}

impl<'a> PuzzlesApi<'a> {
    /// Binds the accessor to a client.
    pub(crate) fn new(client: &'a LichessClient) -> Self {
        Self { client }
    }

    /// Gets the daily puzzle. `GET /api/puzzle/daily`
    pub async fn daily(&self) -> Result<LichessPuzzleAndGame> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/puzzle/daily");
        http::json(request, "LichessPuzzleAndGame").await
    }

    /// Gets a puzzle by id. `GET /api/puzzle/{id}`
    pub async fn get(&self, id: &str) -> Result<LichessPuzzleAndGame> {
        let path = format!("/api/puzzle/{id}");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "LichessPuzzleAndGame").await
    }

    /// Gets a new puzzle, optionally filtered by theme, difficulty, and color.
    ///
    /// `GET /api/puzzle/next`
    pub async fn next(
        &self,
        angle: Option<&str>,
        difficulty: Option<&str>,
        color: Option<&str>,
    ) -> Result<LichessPuzzleAndGame> {
        let query = NextQuery {
            angle,
            difficulty,
            color,
        };
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/puzzle/next")
            .query(&query);
        http::json(request, "LichessPuzzleAndGame").await
    }

    /// Streams the authenticated user's puzzle activity, most recent first.
    ///
    /// Requires the `puzzle:read` scope. `GET /api/puzzle/activity`
    pub async fn activity(
        &self,
        max: Option<u32>,
    ) -> Result<BoxStream<'static, Result<LichessPuzzleActivity>>> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/puzzle/activity")
            .query(&ActivityQuery { max });
        http::stream(request).await
    }

    /// Gets a batch of puzzles for the given angle (theme/opening).
    ///
    /// `GET /api/puzzle/batch/{angle}`
    pub async fn batch(&self, angle: &str, nb: u32) -> Result<LichessPuzzleBatch> {
        let path = format!("/api/puzzle/batch/{angle}");
        let request = self
            .client
            .request(Method::GET, Host::Default, &path)
            .query(&[("nb", nb)]);
        http::json(request, "LichessPuzzleBatch").await
    }

    /// Submits puzzle solutions and gets the next batch.
    ///
    /// `POST /api/puzzle/batch/{angle}`
    pub async fn solve_batch(
        &self,
        angle: &str,
        solutions: &[LichessPuzzleSolution],
    ) -> Result<LichessPuzzleBatch> {
        let path = format!("/api/puzzle/batch/{angle}");
        let body = serde_json::json!({ "solutions": solutions });
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .json(&body);
        http::json(request, "LichessPuzzleBatch").await
    }

    /// Gets the authenticated user's puzzle dashboard for the last `days` days.
    ///
    /// `GET /api/puzzle/dashboard/{days}`
    pub async fn dashboard(&self, days: u32) -> Result<LichessPuzzleDashboard> {
        let path = format!("/api/puzzle/dashboard/{days}");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "LichessPuzzleDashboard").await
    }

    /// Gets a puzzle replay session for a theme over the last `days` days.
    ///
    /// `GET /api/puzzle/replay/{days}/{theme}`
    pub async fn replay(&self, days: u32, theme: &str) -> Result<LichessPuzzleReplay> {
        let path = format!("/api/puzzle/replay/{days}/{theme}");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "LichessPuzzleReplay").await
    }

    /// Gets a user's Puzzle Storm dashboard.
    ///
    /// `GET /api/storm/dashboard/{username}`
    pub async fn storm_dashboard(&self, username: &str) -> Result<LichessStormDashboard> {
        let path = format!("/api/storm/dashboard/{username}");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "LichessStormDashboard").await
    }

    /// Gets a puzzle race by id. `GET /api/racer/{id}`
    pub async fn racer(&self, id: &str) -> Result<LichessPuzzleRacer> {
        let path = format!("/api/racer/{id}");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "LichessPuzzleRacer").await
    }

    /// Creates a new puzzle race. `POST /api/racer`
    pub async fn create_racer(&self) -> Result<LichessPuzzleRacer> {
        let request = self
            .client
            .request(Method::POST, Host::Default, "/api/racer");
        http::json(request, "LichessPuzzleRacer").await
    }
}

impl LichessClient {
    /// Puzzles API: the daily puzzle, lookups, and activity.
    #[must_use]
    pub fn puzzles(&self) -> PuzzlesApi<'_> {
        PuzzlesApi::new(self)
    }
}
