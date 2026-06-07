//! Error types for the litchee client.
//!
//! Every failure mode is mapped to a specific, matchable variant so callers can
//! react to *what* went wrong, not merely *that* something did. The top-level
//! [`LichessError`] aggregates a structured [`ApiError`] (status + kind + body)
//! alongside transport, body-decode, and request-validation failures. Further
//! variants (OAuth, streaming, PKCE) are introduced with the features that use
//! them.

mod api;
mod oauth;
mod pkce;
mod stream;

pub use api::{ApiError, ApiErrorKind};
pub use oauth::{OAuthError, OAuthErrorCode};
pub use pkce::PkceError;
pub use stream::StreamError;

/// The result type returned by all fallible client operations.
pub type Result<T> = std::result::Result<T, LichessError>;

/// The unified error type for every operation in the crate.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum LichessError {
    /// A structured error response from the Lichess HTTP API.
    #[error(transparent)]
    Api(#[from] ApiError),

    /// A typed error from the `OAuth2` token endpoint.
    #[error(transparent)]
    OAuth(#[from] OAuthError),

    /// A transport-level failure (DNS, connection, TLS, timeout).
    #[error("HTTP transport error")]
    Transport(#[from] reqwest::Error),

    /// A successful response whose body could not be deserialized.
    #[error("failed to decode {context}")]
    Decode {
        /// What was being decoded (e.g. `"LichessUser"`), for context.
        context: String,
        /// The underlying deserialization error.
        #[source]
        source: serde_json::Error,
    },

    /// A failure while consuming a streaming (NDJSON) response.
    #[error(transparent)]
    Stream(#[from] StreamError),

    /// The request was rejected client-side before being sent (e.g. a builder
    /// was given values the API does not permit).
    #[error("invalid request: {0}")]
    InvalidRequest(String),

    /// A PKCE parameter was invalid.
    #[error(transparent)]
    Pkce(#[from] PkceError),
}

impl LichessError {
    /// Wraps a body-deserialization failure with human-readable context.
    pub(crate) fn decode(context: impl Into<String>, source: serde_json::Error) -> Self {
        Self::Decode {
            context: context.into(),
            source,
        }
    }
}
