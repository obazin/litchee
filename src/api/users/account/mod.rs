//! The Account API: the authenticated user's own profile and settings.
//!
//! Reached through [`LichessClient::account`].

use reqwest::Method;
use serde::Deserialize;

use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;
use crate::model::LichessUserExtended;

mod model;

pub use model::{
    LichessPreferences, LichessTimeline, LichessTimelineEntry, LichessUserPreferences,
};

/// Internal decode shape for `GET /api/account/email`.
#[derive(Debug, Deserialize)]
struct EmailResponse {
    email: String,
}

/// Internal decode shape for `GET /api/account/kid`.
#[derive(Debug, Deserialize)]
struct KidStatus {
    kid: bool,
}

/// Accessor for the Account API.
///
/// Obtain one via [`LichessClient::account`]. Every method requires an
/// authenticated client (a token set on the builder).
#[derive(Debug)]
pub struct AccountApi<'a> {
    client: &'a LichessClient,
}

impl<'a> AccountApi<'a> {
    /// Binds the accessor to a client.
    pub(crate) fn new(client: &'a LichessClient) -> Self {
        Self { client }
    }

    /// Gets the public profile of the authenticated user.
    ///
    /// `GET /api/account`
    pub async fn profile(&self) -> Result<LichessUserExtended> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/account");
        http::json(request, "LichessUserExtended").await
    }

    /// Gets the email address of the authenticated user.
    ///
    /// Requires the `email:read` scope. `GET /api/account/email`
    pub async fn email(&self) -> Result<String> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/account/email");
        let response: EmailResponse = http::json(request, "account email").await?;
        Ok(response.email)
    }

    /// Reads whether kid mode is enabled for the authenticated user.
    ///
    /// Requires the `preference:read` scope. `GET /api/account/kid`
    pub async fn kid_mode(&self) -> Result<bool> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/account/kid");
        let status: KidStatus = http::json(request, "kid mode status").await?;
        Ok(status.kid)
    }

    /// Enables or disables kid mode for the authenticated user.
    ///
    /// Requires the `preference:write` scope. `POST /api/account/kid`
    pub async fn set_kid_mode(&self, enabled: bool) -> Result<()> {
        let request = self
            .client
            .request(Method::POST, Host::Default, "/api/account/kid")
            .query(&[("v", enabled)]);
        http::ok(request).await
    }

    /// Gets the authenticated user's preferences.
    ///
    /// Requires the `preference:read` scope. `GET /api/account/preferences`
    pub async fn preferences(&self) -> Result<LichessPreferences> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/account/preferences");
        http::json(request, "LichessPreferences").await
    }

    /// Gets the authenticated user's timeline.
    ///
    /// `GET /api/timeline`
    pub async fn timeline(&self) -> Result<LichessTimeline> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/timeline");
        http::json(request, "LichessTimeline").await
    }
}

impl LichessClient {
    /// Account API: the authenticated user's profile and settings.
    #[must_use]
    pub fn account(&self) -> AccountApi<'_> {
        AccountApi::new(self)
    }
}
