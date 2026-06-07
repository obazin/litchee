//! Builders for creating and editing broadcast tournaments and rounds.

use reqwest::Method;
use serde::Serialize;

use super::model::{LichessBroadcast, LichessBroadcastRoundView};
use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;

/// Form body for creating/editing a broadcast tournament.
#[derive(Debug, Serialize)]
struct TourForm<'a> {
    name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    info: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    visibility: Option<&'a str>,
}

/// Builder for creating or editing a broadcast tournament.
#[derive(Debug)]
pub struct TourRequest<'a> {
    client: &'a LichessClient,
    edit_id: Option<&'a str>,
    form: TourForm<'a>,
}

impl<'a> TourRequest<'a> {
    /// Creates the request builder.
    pub(crate) fn new(client: &'a LichessClient, edit_id: Option<&'a str>, name: &'a str) -> Self {
        Self {
            client,
            edit_id,
            form: TourForm {
                name,
                info: None,
                visibility: None,
            },
        }
    }

    /// Sets the short description shown on the broadcast.
    #[must_use]
    pub fn info(mut self, info: &'a str) -> Self {
        self.form.info = Some(info);
        self
    }

    /// Sets the visibility (`public`, `unlisted`, or `private`).
    #[must_use]
    pub fn visibility(mut self, visibility: &'a str) -> Self {
        self.form.visibility = Some(visibility);
        self
    }

    /// Creates or updates the tournament.
    pub async fn send(self) -> Result<LichessBroadcast> {
        let path = match self.edit_id {
            Some(id) => format!("/broadcast/{id}/edit"),
            None => "/broadcast/new".to_owned(),
        };
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .form(&self.form);
        http::json(request, "LichessBroadcast").await
    }
}

/// Form body for creating/editing a broadcast round.
#[derive(Debug, Serialize)]
struct RoundForm<'a> {
    name: &'a str,
    #[serde(rename = "syncUrl", skip_serializing_if = "Option::is_none")]
    sync_url: Option<&'a str>,
    #[serde(rename = "startsAt", skip_serializing_if = "Option::is_none")]
    starts_at: Option<i64>,
}

/// Builder for creating a round (under a tournament) or editing a round.
#[derive(Debug)]
pub struct RoundRequest<'a> {
    client: &'a LichessClient,
    /// Tournament id when creating, or round id when editing.
    target_id: &'a str,
    edit: bool,
    form: RoundForm<'a>,
}

impl<'a> RoundRequest<'a> {
    /// Creates the request builder.
    pub(crate) fn new(
        client: &'a LichessClient,
        target_id: &'a str,
        edit: bool,
        name: &'a str,
    ) -> Self {
        Self {
            client,
            target_id,
            edit,
            form: RoundForm {
                name,
                sync_url: None,
                starts_at: None,
            },
        }
    }

    /// Sets a source URL to automatically sync games from.
    #[must_use]
    pub fn sync_url(mut self, url: &'a str) -> Self {
        self.form.sync_url = Some(url);
        self
    }

    /// Sets the round start time (Unix milliseconds).
    #[must_use]
    pub fn starts_at(mut self, timestamp: i64) -> Self {
        self.form.starts_at = Some(timestamp);
        self
    }

    /// Creates or updates the round.
    pub async fn send(self) -> Result<LichessBroadcastRoundView> {
        let path = if self.edit {
            format!("/broadcast/round/{}/edit", self.target_id)
        } else {
            format!("/broadcast/{}/new", self.target_id)
        };
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .form(&self.form);
        http::json(request, "LichessBroadcastRoundView").await
    }
}
