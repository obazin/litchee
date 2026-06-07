//! The access token returned by the `OAuth2` token endpoint.

use std::fmt;

use serde::Deserialize;

/// An `OAuth2` access token obtained via the PKCE flow.
///
/// The [`Debug`] implementation redacts the secret `access_token`.
#[derive(Clone, Deserialize)]
#[non_exhaustive]
pub struct LichessToken {
    /// The bearer token to authenticate subsequent requests.
    pub access_token: String,
    /// The token type; always `"Bearer"`.
    pub token_type: String,
    /// Lifetime of the token in seconds.
    pub expires_in: u64,
}

impl fmt::Debug for LichessToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LichessToken")
            .field("access_token", &"<redacted>")
            .field("token_type", &self.token_type)
            .field("expires_in", &self.expires_in)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_token_response() {
        let json = r#"{"token_type":"Bearer","access_token":"lio_secret","expires_in":31536000}"#;
        let token: LichessToken = serde_json::from_str(json).unwrap();
        assert_eq!(token.token_type, "Bearer");
        assert_eq!(token.expires_in, 31_536_000);
        assert_eq!(token.access_token, "lio_secret");
    }

    #[test]
    fn debug_redacts_the_secret() {
        let token = LichessToken {
            access_token: "lio_secret".to_owned(),
            token_type: "Bearer".to_owned(),
            expires_in: 10,
        };
        assert!(!format!("{token:?}").contains("lio_secret"));
    }
}
