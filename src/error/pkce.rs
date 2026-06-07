//! Client-side errors from constructing or validating a PKCE flow.

/// An error raised while building or validating PKCE parameters.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum PkceError {
    /// The code verifier length is outside the RFC 7636 range of `43..=128`.
    #[error("invalid code verifier length: {0} (must be between 43 and 128)")]
    InvalidVerifierLength(usize),

    /// The code verifier contains characters outside the allowed alphabet
    /// (`A-Z`, `a-z`, `0-9`, `-`, `.`, `_`, `~`).
    #[error("code verifier contains characters outside the unreserved set")]
    InvalidVerifierChars,
}
