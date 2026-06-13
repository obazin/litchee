//! The Arena Tournaments API: create, run, join, and export arena tournaments.
//!
//! Reached through [`LichessClient::arena`].

use futures_util::stream::BoxStream;
use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::api::gameplay::games::LichessGame;
use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;
use crate::model::{LichessLightUser, LichessTitle, LichessVariantKey};

/// Form body for creating an arena tournament.
#[derive(Debug, Serialize)]
struct CreateForm<'a> {
    name: &'a str,
    #[serde(rename = "clockTime")]
    clock_time: f32,
    #[serde(rename = "clockIncrement")]
    clock_increment: u32,
    minutes: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    rated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    variant: Option<LichessVariantKey>,
    #[serde(rename = "waitMinutes", skip_serializing_if = "Option::is_none")]
    wait_minutes: Option<u32>,
}

/// Accessor for the Arena Tournaments API.
#[derive(Debug)]
pub struct ArenaApi<'a> {
    client: &'a LichessClient,
}

impl<'a> ArenaApi<'a> {
    /// Binds the accessor to a client.
    pub(crate) fn new(client: &'a LichessClient) -> Self {
        Self { client }
    }

    /// Lists current arena tournaments. `GET /api/tournament`
    pub async fn list(&self) -> Result<LichessArenaList> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/tournament");
        http::json(request, "LichessArenaList").await
    }

    /// Gets full details of a tournament. `GET /api/tournament/{id}`
    pub async fn get(&self, id: &str) -> Result<LichessArenaFull> {
        let path = format!("/api/tournament/{id}");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "LichessArenaFull").await
    }

    /// Creates an arena tournament. `POST /api/tournament`
    #[must_use]
    pub fn create(
        &self,
        name: &'a str,
        clock_time: f32,
        clock_increment: u32,
        minutes: u32,
    ) -> CreateArenaRequest<'a> {
        CreateArenaRequest::new(
            self.client,
            None,
            name,
            clock_time,
            clock_increment,
            minutes,
        )
    }

    /// Updates an existing arena tournament. `POST /api/tournament/{id}`
    #[must_use]
    pub fn update(
        &self,
        id: &'a str,
        name: &'a str,
        clock_time: f32,
        clock_increment: u32,
        minutes: u32,
    ) -> CreateArenaRequest<'a> {
        CreateArenaRequest::new(
            self.client,
            Some(id),
            name,
            clock_time,
            clock_increment,
            minutes,
        )
    }

    /// Configures a team battle. `POST /api/tournament/team-battle/{id}`
    pub async fn setup_team_battle(
        &self,
        id: &str,
        team_ids: &[&str],
        nb_leaders: u32,
    ) -> Result<LichessArenaFull> {
        let path = format!("/api/tournament/team-battle/{id}");
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .form(&[
                ("teams", team_ids.join(",").as_str()),
                ("nbLeaders", nb_leaders.to_string().as_str()),
            ]);
        http::json(request, "LichessArenaFull").await
    }

    /// Gets the team standings of a team-battle arena.
    /// `GET /api/tournament/{id}/teams`
    pub async fn teams(&self, id: &str) -> Result<LichessTeamBattleStandings> {
        let path = format!("/api/tournament/{id}/teams");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "LichessTeamBattleStandings").await
    }

    /// Streams the arenas created by a user (NDJSON).
    /// `GET /api/user/{username}/tournament/created`
    pub async fn created_by(
        &self,
        username: &str,
    ) -> Result<BoxStream<'static, Result<LichessArena>>> {
        let path = format!("/api/user/{username}/tournament/created");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::stream(request).await
    }

    /// Streams the arenas a user has played (NDJSON).
    /// `GET /api/user/{username}/tournament/played`
    pub async fn played_by(
        &self,
        username: &str,
    ) -> Result<BoxStream<'static, Result<LichessArena>>> {
        let path = format!("/api/user/{username}/tournament/played");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::stream(request).await
    }

    /// Joins a tournament. `POST /api/tournament/{id}/join`
    pub async fn join(&self, id: &str) -> Result<()> {
        let path = format!("/api/tournament/{id}/join");
        http::ok(self.client.request(Method::POST, Host::Default, &path)).await
    }

    /// Withdraws from a tournament. `POST /api/tournament/{id}/withdraw`
    pub async fn withdraw(&self, id: &str) -> Result<()> {
        let path = format!("/api/tournament/{id}/withdraw");
        http::ok(self.client.request(Method::POST, Host::Default, &path)).await
    }

    /// Terminates a tournament you created. `POST /api/tournament/{id}/terminate`
    pub async fn terminate(&self, id: &str) -> Result<()> {
        let path = format!("/api/tournament/{id}/terminate");
        http::ok(self.client.request(Method::POST, Host::Default, &path)).await
    }

    /// Streams a tournament's results. `GET /api/tournament/{id}/results`
    pub async fn results(
        &self,
        id: &str,
    ) -> Result<BoxStream<'static, Result<LichessArenaResult>>> {
        let path = format!("/api/tournament/{id}/results");
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::stream(request).await
    }

    /// Streams a tournament's games as NDJSON. `GET /api/tournament/{id}/games`
    pub async fn games(&self, id: &str) -> Result<BoxStream<'static, Result<LichessGame>>> {
        let path = format!("/api/tournament/{id}/games");
        let request = self
            .client
            .request(Method::GET, Host::Default, &path)
            .header(reqwest::header::ACCEPT, "application/x-ndjson");
        http::stream(request).await
    }
}

/// Builder for creating or updating an arena tournament.
#[derive(Debug)]
pub struct CreateArenaRequest<'a> {
    client: &'a LichessClient,
    id: Option<&'a str>,
    form: CreateForm<'a>,
}

impl<'a> CreateArenaRequest<'a> {
    /// Creates the request builder.
    fn new(
        client: &'a LichessClient,
        id: Option<&'a str>,
        name: &'a str,
        clock_time: f32,
        clock_increment: u32,
        minutes: u32,
    ) -> Self {
        Self {
            client,
            id,
            form: CreateForm {
                name,
                clock_time,
                clock_increment,
                minutes,
                rated: None,
                variant: None,
                wait_minutes: None,
            },
        }
    }

    /// Sets whether the tournament is rated.
    #[must_use]
    pub fn rated(mut self, rated: bool) -> Self {
        self.form.rated = Some(rated);
        self
    }

    /// Sets the variant.
    #[must_use]
    pub fn variant(mut self, variant: LichessVariantKey) -> Self {
        self.form.variant = Some(variant);
        self
    }

    /// Sets how many minutes to wait before starting.
    #[must_use]
    pub fn wait_minutes(mut self, minutes: u32) -> Self {
        self.form.wait_minutes = Some(minutes);
        self
    }

    /// Creates or updates the tournament.
    pub async fn send(self) -> Result<LichessArenaFull> {
        let path = match self.id {
            Some(id) => format!("/api/tournament/{id}"),
            None => "/api/tournament".to_owned(),
        };
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .form(&self.form);
        http::json(request, "LichessArenaFull").await
    }
}

impl LichessClient {
    /// Arena Tournaments API.
    #[must_use]
    pub fn arena(&self) -> ArenaApi<'_> {
        ArenaApi::new(self)
    }
}

/// An arena clock (`limit` + `increment`, in seconds).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessArenaClock {
    /// Initial time in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Increment per move in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub increment: Option<u32>,
}

/// Perf display info for an arena.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessArenaPerf {
    /// The perf key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// The perf name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The perf icon.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

/// A summary of an arena tournament.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessArena {
    /// The tournament id.
    pub id: String,
    /// The full display name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_name: Option<String>,
    /// The creator's username.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    /// The tournament duration in minutes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minutes: Option<u32>,
    /// The clock.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clock: Option<LichessArenaClock>,
    /// Whether the tournament is rated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rated: Option<bool>,
    /// The number of players.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nb_players: Option<u32>,
    /// Start time (Unix milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub starts_at: Option<i64>,
    /// Finish time (Unix milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finishes_at: Option<i64>,
    /// The status code (10 = created, 20 = started, 30 = finished).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<i32>,
    /// Perf display info.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub perf: Option<LichessArenaPerf>,
    /// Seconds until the tournament starts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seconds_to_start: Option<i64>,
    /// The winner, once finished.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub winner: Option<LichessLightUser>,
}

/// Tournaments grouped by lifecycle stage. `GET /api/tournament`
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessArenaList {
    /// Tournaments not yet started.
    #[serde(default)]
    pub created: Vec<LichessArena>,
    /// Tournaments in progress.
    #[serde(default)]
    pub started: Vec<LichessArena>,
    /// Recently finished tournaments.
    #[serde(default)]
    pub finished: Vec<LichessArena>,
}

/// A player row in an arena standings page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessArenaPlayer {
    /// The player's username.
    pub name: String,
    /// The player's title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<LichessTitle>,
    /// The player's current rank.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rank: Option<u32>,
    /// The player's rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rating: Option<u32>,
    /// The player's score.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<u32>,
}

/// A page of arena standings.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessArenaStanding {
    /// The page number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// The players on this page.
    #[serde(default)]
    pub players: Vec<LichessArenaPlayer>,
}

/// Full details of an arena tournament. `GET /api/tournament/{id}`
///
/// Models the commonly-used fields; deeper aggregates (duels, stats, podium)
/// are not decoded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessArenaFull {
    /// The tournament id.
    pub id: String,
    /// The full display name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_name: Option<String>,
    /// Whether the tournament is rated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rated: Option<bool>,
    /// The clock.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clock: Option<LichessArenaClock>,
    /// The duration in minutes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minutes: Option<u32>,
    /// The creator's username.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    /// Seconds until the tournament starts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seconds_to_start: Option<i64>,
    /// Seconds until the tournament finishes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seconds_to_finish: Option<i64>,
    /// Whether the tournament has finished.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_finished: Option<bool>,
    /// Whether pairings are closed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pairings_closed: Option<bool>,
    /// Start time (Unix milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub starts_at: Option<i64>,
    /// The number of players.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nb_players: Option<u32>,
    /// Perf display info.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub perf: Option<LichessArenaPerf>,
    /// The current standings page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub standing: Option<LichessArenaStanding>,
    /// The authenticated user's username, if entered.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub my_username: Option<String>,
}

/// One entry in an arena results stream.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessArenaResult {
    /// The player's final rank.
    pub rank: u32,
    /// The player's score.
    pub score: u32,
    /// The player's rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rating: Option<u32>,
    /// The player's username.
    pub username: String,
    /// The player's title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<LichessTitle>,
    /// The player's tournament performance rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub performance: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_arena_list() {
        let json = r#"{"created":[],"started":[{"id":"abc","fullName":"Hourly",
            "clock":{"limit":300,"increment":0},"nbPlayers":50,"status":20}],
            "finished":[]}"#;
        let list: LichessArenaList = serde_json::from_str(json).unwrap();
        assert_eq!(list.started.len(), 1);
        assert_eq!(list.started[0].nb_players, Some(50));
    }

    #[test]
    fn parses_full_arena_ignoring_unknown_aggregates() {
        let json = r#"{"id":"abc","fullName":"Hourly","nbPlayers":50,
            "standing":{"page":1,"players":[{"name":"A","rank":1,"score":10}]},
            "duels":[{"whatever":true}],"stats":{"games":100}}"#;
        let full: LichessArenaFull = serde_json::from_str(json).unwrap();
        assert_eq!(full.standing.unwrap().players[0].name, "A");
    }
}

/// A team's standing in a team-battle arena.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessTeamBattleTeam {
    /// The team's rank.
    pub rank: u32,
    /// The team id.
    pub id: String,
    /// The team's score.
    pub score: u32,
}

/// The team standings of a team-battle arena.
/// `GET /api/tournament/{id}/teams`
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessTeamBattleStandings {
    /// The tournament id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The teams, best first.
    #[serde(default)]
    pub teams: Vec<LichessTeamBattleTeam>,
}
