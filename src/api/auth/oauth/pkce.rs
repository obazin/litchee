//! PKCE (Proof Key for Code Exchange, RFC 7636) helpers.

use std::fmt;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::Rng;
use rand::distributions::Alphanumeric;
use sha2::{Digest, Sha256};

use crate::error::PkceError;
use crate::secret::REDACTED;

/// Length of a generated verifier (within the RFC 7636 range of `43..=128`).
const GENERATED_LEN: usize = 64;
/// Minimum RFC 7636 code-verifier length.
const MIN_LEN: usize = 43;
/// Maximum RFC 7636 code-verifier length.
const MAX_LEN: usize = 128;

/// A PKCE code verifier: a high-entropy secret kept by the client between
/// building the authorization URL and exchanging the authorization code.
///
/// Store it securely (e.g. in session state) and never expose it to third
/// parties. Its [`Debug`] output is redacted.
#[derive(Clone)]
pub struct PkceVerifier(String);

impl PkceVerifier {
    /// Generates a fresh random verifier from the unreserved alphabet.
    #[must_use]
    pub fn generate() -> Self {
        Self(random_alphanumeric(GENERATED_LEN))
    }

    /// Wraps an existing verifier string, validating its length and alphabet.
    ///
    /// # Errors
    /// Returns [`PkceError`] if the length is outside `43..=128` or the string
    /// contains characters outside the unreserved set.
    pub fn new(value: impl Into<String>) -> Result<Self, PkceError> {
        let value = value.into();
        if !(MIN_LEN..=MAX_LEN).contains(&value.len()) {
            return Err(PkceError::InvalidVerifierLength(value.len()));
        }
        if !value.bytes().all(is_unreserved) {
            return Err(PkceError::InvalidVerifierChars);
        }
        Ok(Self(value))
    }

    /// The verifier as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Computes the S256 code challenge: `BASE64URL(SHA256(verifier))`.
    #[must_use]
    pub fn code_challenge(&self) -> String {
        let digest = Sha256::digest(self.0.as_bytes());
        URL_SAFE_NO_PAD.encode(digest)
    }
}

impl fmt::Debug for PkceVerifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("PkceVerifier").field(&REDACTED).finish()
    }
}

/// Whether a byte is in the RFC 7636 unreserved set.
fn is_unreserved(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~')
}

/// Generates a random string of `len` ASCII alphanumeric characters.
///
/// Shared by verifier generation and the OAuth `state` value.
pub(crate) fn random_alphanumeric(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_rfc7636_appendix_b_vector() {
        // The reference verifier/challenge pair from RFC 7636, Appendix B.
        let verifier = PkceVerifier::new("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk").unwrap();
        assert_eq!(
            verifier.code_challenge(),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
    }

    #[test]
    fn generated_verifier_is_valid_length_and_alphabet() {
        let verifier = PkceVerifier::generate();
        assert_eq!(verifier.as_str().len(), GENERATED_LEN);
        assert!(PkceVerifier::new(verifier.as_str().to_owned()).is_ok());
    }

    #[test]
    fn rejects_short_verifier() {
        let err = PkceVerifier::new("too-short").unwrap_err();
        assert!(matches!(err, PkceError::InvalidVerifierLength(9)));
    }

    #[test]
    fn rejects_illegal_characters() {
        let value = "a".repeat(50) + " spaces not allowed in here padded out";
        let err = PkceVerifier::new(value).unwrap_err();
        assert!(matches!(err, PkceError::InvalidVerifierChars));
    }

    #[test]
    fn debug_is_redacted() {
        let verifier = PkceVerifier::new("a".repeat(50)).unwrap();
        assert!(!format!("{verifier:?}").contains(&"a".repeat(50)));
    }

    #[test]
    fn random_alphanumeric_has_requested_length() {
        let value = random_alphanumeric(24);
        assert_eq!(value.len(), 24);
        assert!(value.bytes().all(is_unreserved));
    }
}
