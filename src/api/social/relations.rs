//! The Relations API: following and blocking other players.
//!
//! Reached through [`LichessClient::relations`].

use futures_util::stream::BoxStream;
use reqwest::Method;

use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;
use crate::model::LichessUser;

/// Accessor for the Relations API.
#[derive(Debug)]
pub struct RelationsApi<'a> {
    client: &'a LichessClient,
}

impl<'a> RelationsApi<'a> {
    /// Binds the accessor to a client.
    pub(crate) fn new(client: &'a LichessClient) -> Self {
        Self { client }
    }

    /// Streams the users followed by the authenticated user.
    ///
    /// Requires the `follow:read` scope. `GET /api/rel/following`
    pub async fn following(&self) -> Result<BoxStream<'static, Result<LichessUser>>> {
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/rel/following");
        http::stream(request).await
    }

    /// Follows a player. `POST /api/rel/follow/{username}`
    pub async fn follow(&self, username: &str) -> Result<()> {
        self.post_relation("follow", username).await
    }

    /// Unfollows a player. `POST /api/rel/unfollow/{username}`
    pub async fn unfollow(&self, username: &str) -> Result<()> {
        self.post_relation("unfollow", username).await
    }

    /// Blocks a player. `POST /api/rel/block/{username}`
    pub async fn block(&self, username: &str) -> Result<()> {
        self.post_relation("block", username).await
    }

    /// Unblocks a player. `POST /api/rel/unblock/{username}`
    pub async fn unblock(&self, username: &str) -> Result<()> {
        self.post_relation("unblock", username).await
    }

    /// Issues a `POST /api/rel/{action}/{username}` relation change.
    async fn post_relation(&self, action: &str, username: &str) -> Result<()> {
        let path = format!(
            "/api/rel/{}/{}",
            http::segment(action),
            http::segment(username)
        );
        http::ok(self.client.request(Method::POST, Host::Default, &path)).await
    }
}

impl LichessClient {
    /// Relations API: following and blocking other players.
    #[must_use]
    pub fn relations(&self) -> RelationsApi<'_> {
        RelationsApi::new(self)
    }
}
