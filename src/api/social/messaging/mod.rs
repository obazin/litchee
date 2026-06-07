//! The Messaging API: send private messages.
//!
//! Reached through [`LichessClient::messaging`].

use reqwest::Method;

use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;

/// Accessor for the Messaging API.
#[derive(Debug)]
pub struct MessagingApi<'a> {
    client: &'a LichessClient,
}

impl<'a> MessagingApi<'a> {
    /// Binds the accessor to a client.
    pub(crate) fn new(client: &'a LichessClient) -> Self {
        Self { client }
    }

    /// Sends a private message to another player.
    ///
    /// Requires the `msg:write` scope. `POST /inbox/{username}`
    pub async fn send(&self, username: &str, text: &str) -> Result<()> {
        let path = format!("/inbox/{username}");
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .form(&[("text", text)]);
        http::ok(request).await
    }
}

impl LichessClient {
    /// Messaging API: send private messages to other players.
    #[must_use]
    pub fn messaging(&self) -> MessagingApi<'_> {
        MessagingApi::new(self)
    }
}
