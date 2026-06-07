//! Errors returned by the `OAuth2` token endpoint.

use std::fmt;

/// An error returned by the `/api/token` endpoint during the PKCE flow.
///
/// Mirrors the `OAuthError` schema (`{ error, error_description }`), mapping the
/// RFC 6749 `error` field to a typed [`OAuthErrorCode`].
#[derive(Debug, Clone)]
pub struct OAuthError {
    /// The typed error code.
    pub code: OAuthErrorCode,
    /// The optional human-readable `error_description`.
    pub description: Option<String>,
}

impl OAuthError {
    /// Builds an [`OAuthError`] from the raw `error` string and description.
    pub(crate) fn new(error: &str, description: Option<String>) -> Self {
        Self {
            code: OAuthErrorCode::parse(error),
            description,
        }
    }
}

impl fmt::Display for OAuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OAuth error: {}", self.code)?;
        if let Some(description) = &self.description {
            write!(f, " - {description}")?;
        }
        Ok(())
    }
}

impl std::error::Error for OAuthError {}

/// The RFC 6749 token-endpoint error codes Lichess may return.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum OAuthErrorCode {
    /// The request is missing a parameter or is otherwise malformed.
    InvalidRequest,
    /// The provided authorization grant or `code_verifier` is invalid.
    InvalidGrant,
    /// Client authentication failed.
    InvalidClient,
    /// The client is not authorized to use this grant type.
    UnauthorizedClient,
    /// The grant type is not supported.
    UnsupportedGrantType,
    /// The requested scope is invalid or unknown.
    InvalidScope,
    /// The resource owner denied the request.
    AccessDenied,
    /// Any other (unrecognized) error code, preserved verbatim.
    Other(String),
}

impl OAuthErrorCode {
    /// Parses the RFC 6749 `error` string into a typed code. Pure; unit-tested.
    fn parse(error: &str) -> Self {
        match error {
            "invalid_request" => Self::InvalidRequest,
            "invalid_grant" => Self::InvalidGrant,
            "invalid_client" => Self::InvalidClient,
            "unauthorized_client" => Self::UnauthorizedClient,
            "unsupported_grant_type" => Self::UnsupportedGrantType,
            "invalid_scope" => Self::InvalidScope,
            "access_denied" => Self::AccessDenied,
            other => Self::Other(other.to_owned()),
        }
    }
}

impl fmt::Display for OAuthErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest => f.write_str("invalid_request"),
            Self::InvalidGrant => f.write_str("invalid_grant"),
            Self::InvalidClient => f.write_str("invalid_client"),
            Self::UnauthorizedClient => f.write_str("unauthorized_client"),
            Self::UnsupportedGrantType => f.write_str("unsupported_grant_type"),
            Self::InvalidScope => f.write_str("invalid_scope"),
            Self::AccessDenied => f.write_str("access_denied"),
            Self::Other(code) => f.write_str(code),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_codes() {
        assert_eq!(
            OAuthErrorCode::parse("invalid_grant"),
            OAuthErrorCode::InvalidGrant
        );
        assert_eq!(
            OAuthErrorCode::parse("access_denied"),
            OAuthErrorCode::AccessDenied
        );
    }

    #[test]
    fn preserves_unknown_codes() {
        assert_eq!(
            OAuthErrorCode::parse("some_new_code"),
            OAuthErrorCode::Other("some_new_code".to_owned())
        );
    }

    #[test]
    fn display_round_trips_known_codes() {
        assert_eq!(OAuthErrorCode::InvalidGrant.to_string(), "invalid_grant");
    }
}
