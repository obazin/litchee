//! The access token returned by the `OAuth2` token endpoint.

use serde::Deserialize;

use crate::secret::Secret;

/// An `OAuth2` access token obtained via the PKCE flow.
///
/// The secret `access_token` is a [`Secret`], so it is redacted from the
/// [`Debug`] output. Read it with [`Secret::expose`].
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct LichessToken {
    /// The bearer token to authenticate subsequent requests.
    pub access_token: Secret<String>,
    /// The token type; always `"Bearer"`.
    pub token_type: String,
    /// Lifetime of the token in seconds.
    pub expires_in: u64,
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
        assert_eq!(token.access_token.expose(), "lio_secret");
    }

    #[test]
    fn debug_redacts_the_secret() {
        let token = LichessToken {
            access_token: Secret::new("lio_secret".to_owned()),
            token_type: "Bearer".to_owned(),
            expires_in: 10,
        };
        assert!(!format!("{token:?}").contains("lio_secret"));
    }
}
