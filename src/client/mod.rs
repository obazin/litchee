//! The [`LichessClient`] and its builder.

use std::sync::Arc;
use std::time::Duration;

use reqwest::Method;
use url::Url;

use crate::config::{Config, Host};
use crate::error::Result;
use crate::http::ApiRequest;
use crate::retry::RetryPolicy;
use crate::secret::Secret;

/// Default connection timeout applied to the built-in HTTP client.
const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
/// Default read timeout: the maximum gap between received bytes before a
/// connection is treated as stalled. `reqwest` resets it after each successful
/// read, so it bounds stalls without capping total stream duration.
///
/// Set conservatively at 5 minutes: long enough that even a quiet long-lived
/// stream (whose keep-alive cadence the API does not formally guarantee) is
/// never cut off, while still recovering from a genuinely dead connection.
/// Callers wanting faster failure can lower it via [`read_timeout`].
///
/// [`read_timeout`]: LichessClientBuilder::read_timeout
const DEFAULT_READ_TIMEOUT: Duration = Duration::from_mins(5);

/// An asynchronous handle to the Lichess API.
///
/// Construct one with [`LichessClient::builder`] (to set a token or override
/// hosts) or [`LichessClient::new`] for an anonymous client. The handle is
/// cheap to [`Clone`]: the HTTP client and configuration are shared.
///
/// Endpoint groups are reached through accessor methods (added per concern),
/// for example `client.account()` or `client.users()`.
#[derive(Debug, Clone)]
pub struct LichessClient {
    http: reqwest::Client,
    config: Arc<Config>,
}

impl LichessClient {
    /// Creates an anonymous client with default settings.
    ///
    /// # Panics
    /// Panics if the underlying TLS backend fails to initialise (matching
    /// `reqwest::Client::new`). Use [`builder`](Self::builder) to handle this
    /// fallibly.
    #[must_use]
    pub fn new() -> Self {
        Self::builder()
            .build()
            .expect("default Lichess client should always build")
    }

    /// Starts building a customised client.
    #[must_use]
    pub fn builder() -> LichessClientBuilder {
        LichessClientBuilder::default()
    }

    /// Builds an authenticated request to `host` + `path` (path starts `/`).
    ///
    /// Internal: the single place the bearer token is attached. The request
    /// carries the client's [`RetryPolicy`] so rate-limited calls can retry.
    pub(crate) fn request(&self, method: Method, host: Host, path: &str) -> ApiRequest {
        let url = self.config.url(host, path);
        let builder = self.http.request(method, url);
        let builder = match &self.config.token {
            Some(token) => builder.bearer_auth(token.expose()),
            None => builder,
        };
        ApiRequest::new(builder, self.config.retry_policy)
    }

    /// Builds an absolute URL for a host + path without issuing a request.
    ///
    /// Internal: used to construct user-facing URLs such as the `OAuth2`
    /// authorization endpoint.
    pub(crate) fn absolute_url(&self, host: Host, path: &str) -> String {
        self.config.url(host, path)
    }

    /// The configured maximum buffered NDJSON line length, in bytes.
    pub(crate) fn max_line_bytes(&self) -> usize {
        self.config.max_line_bytes
    }
}

impl Default for LichessClient {
    fn default() -> Self {
        Self::new()
    }
}

/// A builder for [`LichessClient`].
///
/// All hosts default to the public Lichess domains but can be overridden, which
/// is useful for self-hosted instances, `localhost`, or pointing tests at a
/// mock server.
#[derive(Debug, Default)]
pub struct LichessClientBuilder {
    config: Config,
    http: Option<reqwest::Client>,
    connect_timeout: Option<Duration>,
    read_timeout: Option<Duration>,
}

impl LichessClientBuilder {
    /// Sets the `OAuth2` / personal access token sent as a bearer token.
    ///
    /// # Security
    /// The token is sent on every request to the configured hosts. The defaults
    /// are HTTPS; if you override a host to a non-TLS (`http://`) URL via
    /// [`base_url`](Self::base_url) (or the other `*_url` setters), the token is
    /// transmitted unencrypted. Only do so over a trusted channel.
    #[must_use]
    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.config.token = Some(Secret::new(token.into()));
        self
    }

    /// Overrides the `User-Agent` header (ignored if a custom HTTP client is
    /// supplied via [`http_client`](Self::http_client)).
    #[must_use]
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.config.user_agent = user_agent.into();
        self
    }

    /// Supplies a pre-configured `reqwest::Client` (proxies, timeouts, …).
    ///
    /// When set, the [`connect_timeout`](Self::connect_timeout) and
    /// [`read_timeout`](Self::read_timeout) are ignored — configure timeouts on
    /// your own client instead.
    #[must_use]
    pub fn http_client(mut self, http: reqwest::Client) -> Self {
        self.http = Some(http);
        self
    }

    /// Sets the connection timeout for the built-in HTTP client (default 30s).
    ///
    /// Only the connection phase is bounded, so long-lived streaming responses
    /// are unaffected. Ignored if a custom client is supplied via
    /// [`http_client`](Self::http_client).
    #[must_use]
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = Some(timeout);
        self
    }

    /// Sets the read timeout for the built-in HTTP client (default 5 minutes).
    ///
    /// This bounds the gap between received bytes, not the total request time,
    /// so a stalled connection is detected while healthy long-lived streams
    /// (which receive periodic keep-alives) keep running. Ignored if a custom
    /// client is supplied via [`http_client`](Self::http_client).
    #[must_use]
    pub fn read_timeout(mut self, timeout: Duration) -> Self {
        self.read_timeout = Some(timeout);
        self
    }

    /// Sets the maximum bytes buffered for a single NDJSON line (default 16 MiB).
    ///
    /// A streaming response whose line exceeds this without a newline fails with
    /// [`StreamError::LineTooLong`](crate::error::StreamError::LineTooLong),
    /// bounding memory against a malformed or stalled stream. Raise it for an
    /// endpoint with unusually large lines, or lower it to tighten the budget.
    #[must_use]
    pub fn max_line_bytes(mut self, max: usize) -> Self {
        self.config.max_line_bytes = max;
        self
    }

    /// Sets the [`RetryPolicy`] for rate-limited (`429`) requests.
    ///
    /// Retries are off by default; supply a policy to enable them. Only `429`
    /// responses are retried (they were not processed, so retrying is safe);
    /// the advised `Retry-After` is honoured.
    #[must_use]
    pub fn retry_policy(mut self, policy: RetryPolicy) -> Self {
        self.config.retry_policy = policy;
        self
    }

    /// Overrides the main host (`lichess.org`).
    ///
    /// # Security
    /// Intended for localhost, self-hosted instances, or pointing tests at a
    /// mock server. If you pass a non-TLS (`http://`) URL while a
    /// [`token`](Self::token) is set, the bearer token is sent unencrypted —
    /// only do this over a trusted channel.
    #[must_use]
    pub fn base_url(mut self, url: &Url) -> Self {
        self.config.set_base(Host::Default, url);
        self
    }

    /// Overrides the opening-explorer host (`explorer.lichess.org`).
    #[must_use]
    pub fn opening_explorer_url(mut self, url: &Url) -> Self {
        self.config.set_base(Host::OpeningExplorer, url);
        self
    }

    /// Overrides the tablebase host (`tablebase.lichess.org`).
    #[must_use]
    pub fn tablebase_url(mut self, url: &Url) -> Self {
        self.config.set_base(Host::Tablebase, url);
        self
    }

    /// Overrides the external-engine host (`engine.lichess.ovh`).
    #[must_use]
    pub fn engine_url(mut self, url: &Url) -> Self {
        self.config.set_base(Host::Engine, url);
        self
    }

    /// Finishes building the client.
    ///
    /// # Errors
    /// Returns a transport error if a default `reqwest::Client` cannot be built.
    pub fn build(self) -> Result<LichessClient> {
        let http = match self.http {
            Some(http) => http,
            None => reqwest::Client::builder()
                .user_agent(&self.config.user_agent)
                .connect_timeout(self.connect_timeout.unwrap_or(DEFAULT_CONNECT_TIMEOUT))
                .read_timeout(self.read_timeout.unwrap_or(DEFAULT_READ_TIMEOUT))
                .build()?,
        };
        Ok(LichessClient {
            http,
            config: Arc::new(self.config),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_with_custom_read_timeout() {
        let client = LichessClient::builder()
            .read_timeout(Duration::from_secs(10))
            .build();
        assert!(client.is_ok());
    }

    #[test]
    fn max_line_bytes_setter_overrides_the_default() {
        let client = LichessClient::builder()
            .max_line_bytes(1024)
            .build()
            .unwrap();
        assert_eq!(client.max_line_bytes(), 1024);
    }

    #[test]
    fn builds_with_token_and_connect_timeout() {
        let client = LichessClient::builder()
            .token("lip_test")
            .connect_timeout(Duration::from_secs(5))
            .build();
        assert!(client.is_ok());
    }

    #[test]
    fn anonymous_request_has_no_bearer_header() {
        let client = LichessClient::new();
        let request = client
            .request(Method::GET, Host::Default, "/api/account")
            .build()
            .unwrap();
        assert!(
            request
                .headers()
                .get(reqwest::header::AUTHORIZATION)
                .is_none()
        );
    }

    #[test]
    fn authenticated_request_sets_bearer_header() {
        let client = LichessClient::builder().token("lip_test").build().unwrap();
        let request = client
            .request(Method::GET, Host::Default, "/api/account")
            .build()
            .unwrap();
        let auth = request
            .headers()
            .get(reqwest::header::AUTHORIZATION)
            .unwrap();
        assert_eq!(auth, "Bearer lip_test");
    }
}
