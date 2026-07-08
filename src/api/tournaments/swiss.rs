//! The Swiss Tournaments API: create, run, join, and export swiss tournaments.
//!
//! Reached through [`LichessClient::swiss`].

use futures_util::stream::BoxStream;
use reqwest::{Method, StatusCode};
use serde::{Deserialize, Serialize};

use crate::api::gameplay::games::LichessGame;
use crate::client::LichessClient;
use crate::config::Host;
use crate::error::{ApiErrorKind, LichessError, Result};
use crate::http;
use crate::model::{GameExportOptions, LichessTitle, LichessVariantKey};

/// Reclassifies a `401` from a Swiss edit/schedule request as the distinct
/// [`ApiErrorKind::SwissUnauthorizedEdit`] ownership rejection — the spec's
/// `SwissUnauthorisedEdit` response — rather than a generic auth failure.
fn map_unauthorized_edit(err: LichessError) -> LichessError {
    match err {
        LichessError::Api(api) if api.status == StatusCode::UNAUTHORIZED => {
            LichessError::Api(api.with_kind(ApiErrorKind::SwissUnauthorizedEdit))
        }
        other => other,
    }
}

/// Form body for creating a swiss tournament (flat, non-`conditions` fields).
#[derive(Debug, Default, Serialize)]
struct CreateForm<'a> {
    #[serde(rename = "clock.limit")]
    clock_limit: u32,
    #[serde(rename = "clock.increment")]
    clock_increment: u32,
    #[serde(rename = "nbRounds")]
    nb_rounds: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rated: Option<bool>,
    #[serde(rename = "roundInterval", skip_serializing_if = "Option::is_none")]
    round_interval: Option<u32>,
    #[serde(rename = "startsAt", skip_serializing_if = "Option::is_none")]
    starts_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    variant: Option<LichessVariantKey>,
    #[serde(skip_serializing_if = "Option::is_none")]
    position: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    password: Option<&'a str>,
    #[serde(rename = "forbiddenPairings", skip_serializing_if = "Option::is_none")]
    forbidden_pairings: Option<&'a str>,
    #[serde(rename = "manualPairings", skip_serializing_if = "Option::is_none")]
    manual_pairings: Option<&'a str>,
    #[serde(rename = "chatFor", skip_serializing_if = "Option::is_none")]
    chat_for: Option<u32>,
}

/// Entry conditions for a swiss tournament, serialized as `conditions.*` keys.
#[derive(Debug, Clone, Default, Serialize)]
pub struct SwissConditions<'a> {
    #[serde(
        rename = "conditions.minRating.rating",
        skip_serializing_if = "Option::is_none"
    )]
    min_rating: Option<u32>,
    #[serde(
        rename = "conditions.maxRating.rating",
        skip_serializing_if = "Option::is_none"
    )]
    max_rating: Option<u32>,
    #[serde(
        rename = "conditions.nbRatedGame.nb",
        skip_serializing_if = "Option::is_none"
    )]
    nb_rated_games: Option<u32>,
    #[serde(
        rename = "conditions.allowList",
        skip_serializing_if = "Option::is_none"
    )]
    allow_list: Option<&'a str>,
    #[serde(
        rename = "conditions.playYourGames",
        skip_serializing_if = "Option::is_none"
    )]
    play_your_games: Option<bool>,
}

impl<'a> SwissConditions<'a> {
    /// Minimum rating to join.
    #[must_use]
    pub fn min_rating(mut self, rating: u32) -> Self {
        self.min_rating = Some(rating);
        self
    }

    /// Maximum rating to join.
    #[must_use]
    pub fn max_rating(mut self, rating: u32) -> Self {
        self.max_rating = Some(rating);
        self
    }

    /// Minimum number of rated games required to join.
    #[must_use]
    pub fn nb_rated_games(mut self, count: u32) -> Self {
        self.nb_rated_games = Some(count);
        self
    }

    /// Comma-separated usernames allowed to join (append `%titled` to also allow
    /// any titled player).
    #[must_use]
    pub fn allow_list(mut self, usernames: &'a str) -> Self {
        self.allow_list = Some(usernames);
        self
    }

    /// Require players to have played their games (no repeated no-shows).
    #[must_use]
    pub fn play_your_games(mut self, value: bool) -> Self {
        self.play_your_games = Some(value);
        self
    }
}

/// Accessor for the Swiss Tournaments API.
#[derive(Debug)]
pub struct SwissApi<'a> {
    client: &'a LichessClient,
}

impl<'a> SwissApi<'a> {
    /// Binds the accessor to a client.
    pub(crate) fn new(client: &'a LichessClient) -> Self {
        Self { client }
    }

    /// Gets a swiss tournament. `GET /api/swiss/{id}`
    pub async fn get(&self, id: &str) -> Result<LichessSwiss> {
        let path = format!("/api/swiss/{}", http::segment(id));
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::json(request, "LichessSwiss").await
    }

    /// Creates a swiss tournament for a team. `POST /api/swiss/new/{teamId}`
    #[must_use]
    pub fn create(
        &self,
        team_id: &'a str,
        clock_limit: u32,
        clock_increment: u32,
        nb_rounds: u32,
    ) -> CreateSwissRequest<'a> {
        CreateSwissRequest::new(
            self.client,
            team_id,
            false,
            clock_limit,
            clock_increment,
            nb_rounds,
        )
    }

    /// Updates a swiss tournament. `POST /api/swiss/{id}/edit`
    #[must_use]
    pub fn edit(
        &self,
        id: &'a str,
        clock_limit: u32,
        clock_increment: u32,
        nb_rounds: u32,
    ) -> CreateSwissRequest<'a> {
        CreateSwissRequest::new(
            self.client,
            id,
            true,
            clock_limit,
            clock_increment,
            nb_rounds,
        )
    }

    /// Joins a swiss tournament. `POST /api/swiss/{id}/join`
    pub async fn join(&self, id: &str) -> Result<()> {
        self.post_action(id, "join").await
    }

    /// Withdraws from a swiss tournament. `POST /api/swiss/{id}/withdraw`
    pub async fn withdraw(&self, id: &str) -> Result<()> {
        self.post_action(id, "withdraw").await
    }

    /// Terminates a swiss tournament. `POST /api/swiss/{id}/terminate`
    pub async fn terminate(&self, id: &str) -> Result<()> {
        self.post_action(id, "terminate").await
    }

    /// Manually schedules the next round.
    /// `POST /api/swiss/{id}/schedule-next-round`
    pub async fn schedule_next_round(&self, id: &str) -> Result<()> {
        self.post_action(id, "schedule-next-round")
            .await
            .map_err(map_unauthorized_edit)
    }

    /// Downloads the tournament in TRF format. `GET /swiss/{id}.trf`
    pub async fn trf(&self, id: &str) -> Result<String> {
        let path = format!("/swiss/{}.trf", http::segment(id));
        http::text(self.client.request(Method::GET, Host::Default, &path)).await
    }

    /// Streams a swiss tournament's results. `GET /api/swiss/{id}/results`
    pub async fn results(
        &self,
        id: &str,
    ) -> Result<BoxStream<'static, Result<LichessSwissResult>>> {
        let path = format!("/api/swiss/{}/results", http::segment(id));
        let request = self.client.request(Method::GET, Host::Default, &path);
        http::stream(request, self.client.max_line_bytes()).await
    }

    /// Starts an export of a swiss tournament's games. `GET /api/swiss/{id}/games`
    ///
    /// Finish with [`stream`](SwissGamesRequest::stream) or
    /// [`pgn`](SwissGamesRequest::pgn).
    #[must_use]
    pub fn games(&self, id: &'a str) -> SwissGamesRequest<'a> {
        SwissGamesRequest::new(self.client, id)
    }

    /// Issues a no-argument `POST` action on a swiss tournament.
    async fn post_action(&self, id: &str, action: &str) -> Result<()> {
        let path = format!("/api/swiss/{}/{}", http::segment(id), http::segment(action));
        http::ok(self.client.request(Method::POST, Host::Default, &path)).await
    }
}

/// Builder for creating or editing a swiss tournament.
#[derive(Debug)]
pub struct CreateSwissRequest<'a> {
    client: &'a LichessClient,
    /// Team id for creation, or tournament id when editing.
    target_id: &'a str,
    edit: bool,
    form: CreateForm<'a>,
    conditions: SwissConditions<'a>,
}

impl<'a> CreateSwissRequest<'a> {
    /// Creates the request builder.
    fn new(
        client: &'a LichessClient,
        target_id: &'a str,
        edit: bool,
        clock_limit: u32,
        clock_increment: u32,
        nb_rounds: u32,
    ) -> Self {
        Self {
            client,
            target_id,
            edit,
            form: CreateForm {
                clock_limit,
                clock_increment,
                nb_rounds,
                ..Default::default()
            },
            conditions: SwissConditions::default(),
        }
    }

    /// Sets the tournament name.
    #[must_use]
    pub fn name(mut self, name: &'a str) -> Self {
        self.form.name = Some(name);
        self
    }

    /// Sets whether the tournament is rated.
    #[must_use]
    pub fn rated(mut self, rated: bool) -> Self {
        self.form.rated = Some(rated);
        self
    }

    /// Sets the interval between rounds, in seconds.
    #[must_use]
    pub fn round_interval(mut self, seconds: u32) -> Self {
        self.form.round_interval = Some(seconds);
        self
    }

    /// Starts the tournament at this timestamp (Unix milliseconds).
    #[must_use]
    pub fn starts_at(mut self, timestamp: i64) -> Self {
        self.form.starts_at = Some(timestamp);
        self
    }

    /// Sets the variant.
    #[must_use]
    pub fn variant(mut self, variant: LichessVariantKey) -> Self {
        self.form.variant = Some(variant);
        self
    }

    /// Sets a custom starting position (FEN).
    #[must_use]
    pub fn position(mut self, fen: &'a str) -> Self {
        self.form.position = Some(fen);
        self
    }

    /// Sets the tournament description.
    #[must_use]
    pub fn description(mut self, description: &'a str) -> Self {
        self.form.description = Some(description);
        self
    }

    /// Makes the tournament private, restricted by this password.
    #[must_use]
    pub fn password(mut self, password: &'a str) -> Self {
        self.form.password = Some(password);
        self
    }

    /// Sets pairings that must not occur (newline-separated username pairs).
    #[must_use]
    pub fn forbidden_pairings(mut self, pairings: &'a str) -> Self {
        self.form.forbidden_pairings = Some(pairings);
        self
    }

    /// Sets manual pairings for the next round (newline-separated username pairs).
    #[must_use]
    pub fn manual_pairings(mut self, pairings: &'a str) -> Self {
        self.form.manual_pairings = Some(pairings);
        self
    }

    /// Sets who may use the chat (Lichess `chatFor` code).
    #[must_use]
    pub fn chat_for(mut self, chat_for: u32) -> Self {
        self.form.chat_for = Some(chat_for);
        self
    }

    /// Sets the entry conditions.
    #[must_use]
    pub fn conditions(mut self, conditions: SwissConditions<'a>) -> Self {
        self.conditions = conditions;
        self
    }

    /// Creates or updates the tournament.
    pub async fn send(self) -> Result<LichessSwiss> {
        let path = self.path();
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .form_parts(&self.form, &self.conditions);
        let result = http::json(request, "LichessSwiss").await;
        if self.edit {
            result.map_err(map_unauthorized_edit)
        } else {
            result
        }
    }

    /// The create or edit path for this request.
    fn path(&self) -> String {
        if self.edit {
            format!("/api/swiss/{}/edit", http::segment(self.target_id))
        } else {
            format!("/api/swiss/new/{}", http::segment(self.target_id))
        }
    }
}

/// Builder for exporting a swiss tournament's games
/// (`GET /api/swiss/{id}/games`).
#[derive(Debug)]
pub struct SwissGamesRequest<'a> {
    client: &'a LichessClient,
    id: &'a str,
    player: Option<&'a str>,
    export: GameExportOptions,
}

impl<'a> SwissGamesRequest<'a> {
    /// Creates the request builder.
    pub(crate) fn new(client: &'a LichessClient, id: &'a str) -> Self {
        Self {
            client,
            id,
            player: None,
            export: GameExportOptions::default(),
        }
    }

    /// Only games featuring this player.
    #[must_use]
    pub fn player(mut self, player: &'a str) -> Self {
        self.player = Some(player);
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
        let request = self.request(http::ACCEPT_NDJSON);
        http::stream(request, self.client.max_line_bytes()).await
    }

    /// Executes the export, returning all games as one PGN string.
    pub async fn pgn(self) -> Result<String> {
        http::text(self.request(http::ACCEPT_PGN)).await
    }

    /// Builds the request with the given `Accept` representation.
    fn request(&self, accept: &'static str) -> http::ApiRequest {
        let path = format!("/api/swiss/{}/games", http::segment(self.id));
        self.client
            .request(Method::GET, Host::Default, &path)
            .header(reqwest::header::ACCEPT, accept)
            .query(&[("player", self.player)])
            .query(&self.export)
    }
}

impl LichessClient {
    /// Swiss Tournaments API.
    #[must_use]
    pub fn swiss(&self) -> SwissApi<'_> {
        SwissApi::new(self)
    }
}

/// A swiss clock (`limit` + `increment`, in seconds).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessSwissClock {
    /// Initial time in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Increment per move in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub increment: Option<u32>,
}

/// When the next round of a swiss starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessSwissNextRound {
    /// Absolute start time (Unix milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub at: Option<i64>,
    /// Seconds until the next round starts.
    #[serde(rename = "in", default, skip_serializing_if = "Option::is_none")]
    pub in_seconds: Option<i64>,
}

/// A swiss tournament.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessSwiss {
    /// The tournament id.
    pub id: String,
    /// The creator's username.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    /// Start time (Unix milliseconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub starts_at: Option<i64>,
    /// The tournament name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The clock.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clock: Option<LichessSwissClock>,
    /// The variant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    /// The current round number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub round: Option<u32>,
    /// The total number of rounds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nb_rounds: Option<u32>,
    /// The number of players.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nb_players: Option<u32>,
    /// The number of ongoing games.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nb_ongoing: Option<u32>,
    /// The status (`created`, `started`, or `finished`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Whether the tournament is rated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rated: Option<bool>,
    /// When the next round starts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_round: Option<LichessSwissNextRound>,
}

/// One entry in a swiss results stream.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessSwissResult {
    /// The player's final rank.
    pub rank: u32,
    /// The player's points.
    pub points: f64,
    /// The player's tie-break score.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tie_break: Option<f64>,
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
    use crate::error::ApiError;

    #[test]
    fn maps_401_to_swiss_unauthorized_edit() {
        let err = LichessError::Api(ApiError::new(StatusCode::UNAUTHORIZED, None, None));
        match map_unauthorized_edit(err) {
            LichessError::Api(api) => assert_eq!(api.kind, ApiErrorKind::SwissUnauthorizedEdit),
            other => panic!("expected Api error, got {other:?}"),
        }
    }

    #[test]
    fn leaves_non_401_errors_unchanged() {
        let err = LichessError::Api(ApiError::new(StatusCode::NOT_FOUND, None, None));
        match map_unauthorized_edit(err) {
            LichessError::Api(api) => assert_eq!(api.kind, ApiErrorKind::NotFound),
            other => panic!("expected Api error, got {other:?}"),
        }
    }

    #[test]
    fn parses_swiss_with_next_round() {
        let json = r#"{"id":"abc","name":"Weekly","clock":{"limit":300,"increment":0},
            "variant":"standard","round":2,"nbRounds":7,"nbPlayers":40,
            "status":"started","rated":true,"nextRound":{"at":1700000000000,"in":120}}"#;
        let swiss: LichessSwiss = serde_json::from_str(json).unwrap();
        assert_eq!(swiss.nb_rounds, Some(7));
        assert_eq!(swiss.next_round.unwrap().in_seconds, Some(120));
    }

    #[test]
    fn parses_swiss_result_with_fractional_points() {
        let json = r#"{"rank":1,"points":5.5,"tieBreak":18.0,"username":"A","rating":2400}"#;
        let result: LichessSwissResult = serde_json::from_str(json).unwrap();
        assert!((result.points - 5.5).abs() < f64::EPSILON);
    }

    #[test]
    fn conditions_serialize_to_dotted_keys() {
        let query = serde_urlencoded::to_string(
            SwissConditions::default()
                .max_rating(2200)
                .play_your_games(true),
        )
        .unwrap();
        assert!(query.contains("conditions.maxRating.rating=2200"));
        assert!(query.contains("conditions.playYourGames=true"));
    }

    #[test]
    fn empty_conditions_serialize_to_nothing() {
        assert_eq!(
            serde_urlencoded::to_string(SwissConditions::default()).unwrap(),
            ""
        );
    }
}
