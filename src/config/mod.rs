//! Connection configuration: the API hosts and per-client settings.
//!
//! Lichess serves its API from several hosts. Most endpoints live on
//! `lichess.org`, but the opening explorer, endgame tablebase, and external
//! engine *work* endpoints each have their own host. [`Host`] selects which one
//! a request targets; [`Config`] holds the resolved base URLs plus the token
//! and user agent.

use url::Url;

use crate::retry::RetryPolicy;
use crate::secret::Secret;

/// The crate's default `User-Agent`, e.g. `litchee/0.1.0`.
pub(crate) const DEFAULT_USER_AGENT: &str = concat!("litchee/", env!("CARGO_PKG_VERSION"));

/// Default base URL for the main Lichess host (`lichess.org`).
const DEFAULT_BASE: &str = "https://lichess.org";
/// Default base URL for the opening-explorer host.
const EXPLORER_BASE: &str = "https://explorer.lichess.org";
/// Default base URL for the tablebase host.
const TABLEBASE_BASE: &str = "https://tablebase.lichess.org";
/// Default base URL for the external-engine host.
const ENGINE_BASE: &str = "https://engine.lichess.ovh";
/// Default cap on a single buffered NDJSON line (see [`Config::max_line_bytes`]).
pub(crate) const DEFAULT_MAX_LINE_BYTES: usize = 16 * 1024 * 1024;

/// One of the hosts the Lichess API is served from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Host {
    /// `lichess.org` — the main API host (the default for nearly everything).
    Default,
    /// `explorer.lichess.org` — the opening explorer.
    OpeningExplorer,
    /// `tablebase.lichess.org` — the endgame tablebase.
    Tablebase,
    /// `engine.lichess.ovh` — external engine work endpoints.
    Engine,
}

/// Resolved per-client configuration shared behind the [`LichessClient`].
///
/// Base URLs are stored without a trailing slash so a path with a leading slash
/// can be appended directly.
///
/// The `token` is a [`Secret`], so it is redacted from the [`Debug`] output and
/// cannot leak through logs that format the client or its builder.
///
/// [`LichessClient`]: crate::LichessClient
#[derive(Debug, Clone)]
pub(crate) struct Config {
    default_base: String,
    explorer_base: String,
    tablebase_base: String,
    engine_base: String,
    pub(crate) token: Option<Secret<String>>,
    pub(crate) user_agent: String,
    /// Maximum bytes buffered for a single NDJSON line before the stream errors
    /// with [`StreamError::LineTooLong`](crate::error::StreamError::LineTooLong),
    /// a guard against unbounded memory growth on a malformed/stalled stream.
    pub(crate) max_line_bytes: usize,
    /// How rate-limited (`429`) requests are retried (default: no retries).
    pub(crate) retry_policy: RetryPolicy,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_base: DEFAULT_BASE.to_owned(),
            explorer_base: EXPLORER_BASE.to_owned(),
            tablebase_base: TABLEBASE_BASE.to_owned(),
            engine_base: ENGINE_BASE.to_owned(),
            token: None,
            user_agent: DEFAULT_USER_AGENT.to_owned(),
            max_line_bytes: DEFAULT_MAX_LINE_BYTES,
            retry_policy: RetryPolicy::default(),
        }
    }
}

impl Config {
    /// Returns the base URL (without trailing slash) for the given host.
    pub(crate) fn base(&self, host: Host) -> &str {
        match host {
            Host::Default => &self.default_base,
            Host::OpeningExplorer => &self.explorer_base,
            Host::Tablebase => &self.tablebase_base,
            Host::Engine => &self.engine_base,
        }
    }

    /// Overrides the base URL for a host. The trailing slash is normalised away
    /// so it can be joined with leading-slash paths.
    pub(crate) fn set_base(&mut self, host: Host, base: &Url) {
        let normalised = base.as_str().trim_end_matches('/').to_owned();
        match host {
            Host::Default => self.default_base = normalised,
            Host::OpeningExplorer => self.explorer_base = normalised,
            Host::Tablebase => self.tablebase_base = normalised,
            Host::Engine => self.engine_base = normalised,
        }
    }

    /// Builds the absolute URL for a host + path (the path must start with `/`).
    pub(crate) fn url(&self, host: Host, path: &str) -> String {
        format!("{}{path}", self.base(host))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_hosts_resolve_to_lichess_domains() {
        let config = Config::default();
        assert_eq!(
            config.url(Host::Default, "/api/account"),
            "https://lichess.org/api/account"
        );
        assert_eq!(
            config.url(Host::OpeningExplorer, "/lichess"),
            "https://explorer.lichess.org/lichess"
        );
        assert_eq!(
            config.url(Host::Tablebase, "/standard"),
            "https://tablebase.lichess.org/standard"
        );
        assert_eq!(
            config.url(Host::Engine, "/api/external-engine/work"),
            "https://engine.lichess.ovh/api/external-engine/work"
        );
    }

    #[test]
    fn debug_redacts_the_token() {
        let config = Config {
            token: Some(Secret::new("lip_supersecret".to_owned())),
            ..Default::default()
        };
        let debug = format!("{config:?}");
        assert!(!debug.contains("lip_supersecret"));
        assert!(debug.contains("<redacted>"));
        // Non-secret fields stay visible.
        assert!(debug.contains("litchee/"));
        assert!(debug.contains("https://lichess.org"));
    }

    #[test]
    fn debug_shows_none_token_as_none() {
        let debug = format!("{:?}", Config::default());
        assert!(debug.contains("token: None"));
    }

    #[test]
    fn set_base_normalises_trailing_slash() {
        let mut config = Config::default();
        let mock = Url::parse("http://127.0.0.1:8080/").unwrap();
        config.set_base(Host::Default, &mock);
        assert_eq!(
            config.url(Host::Default, "/api/account"),
            "http://127.0.0.1:8080/api/account"
        );
    }
}
