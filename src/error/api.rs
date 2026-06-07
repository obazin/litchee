//! HTTP-level API errors returned by Lichess.

use std::fmt;

use reqwest::StatusCode;

/// An error response returned by the Lichess HTTP API.
///
/// Carries the raw [`StatusCode`], a [`kind`](ApiErrorKind) derived from it so
/// callers can match without comparing magic numbers, and the optional
/// human-readable `message` parsed from the `{ "error": ... }` body.
#[derive(Debug, Clone)]
pub struct ApiError {
    /// The HTTP status code of the response.
    pub status: StatusCode,
    /// A matchable classification of the failure.
    pub kind: ApiErrorKind,
    /// The message extracted from the response body, when present.
    pub message: Option<String>,
}

impl ApiError {
    /// Builds an [`ApiError`] from a status code, optional body message, and an
    /// optional `Retry-After` value (seconds) read from the response headers.
    pub(crate) fn new(
        status: StatusCode,
        message: Option<String>,
        retry_after_secs: Option<u64>,
    ) -> Self {
        Self {
            kind: ApiErrorKind::from_status(status, retry_after_secs),
            status,
            message,
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Lichess API error {}: {}",
            self.status.as_u16(),
            self.kind
        )?;
        if let Some(message) = &self.message {
            write!(f, " - {message}")?;
        }
        Ok(())
    }
}

impl std::error::Error for ApiError {}

/// A classification of an [`ApiError`], derived from the HTTP status code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ApiErrorKind {
    /// `400` — the request was malformed or rejected.
    BadRequest,
    /// `401` — authentication is missing or invalid.
    Unauthorized,
    /// `403` — the token lacks the required scope/permission.
    Forbidden,
    /// `404` — the resource does not exist.
    NotFound,
    /// `409` — the request conflicts with the current state.
    Conflict,
    /// `413` — the request payload is too large.
    PayloadTooLarge,
    /// `429` — too many requests; back off before retrying.
    RateLimited {
        /// Seconds to wait before retrying, from the `Retry-After` header.
        retry_after_secs: Option<u64>,
    },
    /// `500`–`599` — a server-side error.
    Server,
    /// Any other status code not otherwise classified.
    Unexpected(u16),
}

impl ApiErrorKind {
    /// Maps an HTTP status code to a kind. Pure; unit-tested.
    fn from_status(status: StatusCode, retry_after_secs: Option<u64>) -> Self {
        match status.as_u16() {
            400 => Self::BadRequest,
            401 => Self::Unauthorized,
            403 => Self::Forbidden,
            404 => Self::NotFound,
            409 => Self::Conflict,
            413 => Self::PayloadTooLarge,
            429 => Self::RateLimited { retry_after_secs },
            500..=599 => Self::Server,
            other => Self::Unexpected(other),
        }
    }
}

impl fmt::Display for ApiErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadRequest => f.write_str("bad request"),
            Self::Unauthorized => f.write_str("unauthorized"),
            Self::Forbidden => f.write_str("forbidden"),
            Self::NotFound => f.write_str("not found"),
            Self::Conflict => f.write_str("conflict"),
            Self::PayloadTooLarge => f.write_str("payload too large"),
            Self::RateLimited {
                retry_after_secs: Some(secs),
            } => write!(f, "rate limited (retry after {secs}s)"),
            Self::RateLimited {
                retry_after_secs: None,
            } => f.write_str("rate limited"),
            Self::Server => f.write_str("server error"),
            Self::Unexpected(code) => write!(f, "unexpected status {code}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kind(code: u16) -> ApiErrorKind {
        ApiErrorKind::from_status(StatusCode::from_u16(code).unwrap(), None)
    }

    #[test]
    fn maps_documented_status_codes() {
        assert_eq!(kind(400), ApiErrorKind::BadRequest);
        assert_eq!(kind(401), ApiErrorKind::Unauthorized);
        assert_eq!(kind(403), ApiErrorKind::Forbidden);
        assert_eq!(kind(404), ApiErrorKind::NotFound);
        assert_eq!(kind(409), ApiErrorKind::Conflict);
        assert_eq!(kind(413), ApiErrorKind::PayloadTooLarge);
    }

    #[test]
    fn maps_server_and_unexpected_ranges() {
        assert_eq!(kind(500), ApiErrorKind::Server);
        assert_eq!(kind(503), ApiErrorKind::Server);
        assert_eq!(kind(418), ApiErrorKind::Unexpected(418));
    }

    #[test]
    fn rate_limit_preserves_retry_after() {
        let limited = ApiErrorKind::from_status(StatusCode::TOO_MANY_REQUESTS, Some(42));
        assert_eq!(
            limited,
            ApiErrorKind::RateLimited {
                retry_after_secs: Some(42)
            }
        );
    }

    #[test]
    fn display_includes_status_and_message() {
        let err = ApiError::new(StatusCode::NOT_FOUND, Some("Not found.".into()), None);
        assert_eq!(
            err.to_string(),
            "Lichess API error 404: not found - Not found."
        );
    }
}
