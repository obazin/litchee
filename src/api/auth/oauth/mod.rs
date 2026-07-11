//! The `OAuth2` Authorization Code flow with PKCE — "Log in with Lichess".
//!
//! Reached through [`LichessClient::oauth`].
//!
//! The flow has two steps:
//!
//! 1. [`OauthApi::authorization_url`] builds the URL to send the user to, along
//!    with the [`PkceVerifier`] and `state` you must store until the redirect
//!    returns.
//! 2. After the redirect, verify `state`, then call
//!    [`OauthApi::exchange_code`] with the returned `code` and your stored
//!    verifier to obtain a [`LichessToken`].
//!
//! [`OauthApi::revoke_token`] revokes the current client's token.

mod pkce;
mod scope;
mod token;

pub use pkce::PkceVerifier;
pub use scope::Scope;
pub use token::LichessToken;

use reqwest::Method;
use serde::Deserialize;
use url::Url;

use crate::client::LichessClient;
use crate::config::Host;
use crate::error::{ApiError, LichessError, OAuthError, Result};
use crate::http;

/// Length of the generated CSRF `state` value.
const STATE_LEN: usize = 24;

/// Parameters for building an authorization URL.
#[derive(Debug, Clone)]
pub struct AuthorizationRequest<'a> {
    /// An identifier that uniquely identifies your application.
    pub client_id: &'a str,
    /// The absolute URL the user is redirected back to.
    pub redirect_uri: &'a str,
    /// The scopes to request (may be empty).
    pub scopes: &'a [Scope],
    /// Optional hint to pre-fill a specific Lichess username on the login page.
    pub username_hint: Option<&'a str>,
}

/// The result of [`OauthApi::authorization_url`].
///
/// Send the user to [`url`](Self::url); persist [`state`](Self::state) and
/// [`verifier`](Self::verifier) until the redirect returns.
#[derive(Debug)]
pub struct Authorization {
    /// The URL to send the user to.
    pub url: Url,
    /// The CSRF `state` returned verbatim with the redirect; verify it matches.
    pub state: String,
    /// The PKCE verifier needed to exchange the authorization code.
    pub verifier: PkceVerifier,
}

/// Parameters for exchanging an authorization code for a token.
#[derive(Debug, Clone)]
pub struct CodeExchange<'a> {
    /// The `code` query parameter received at the redirect URI.
    pub code: &'a str,
    /// The verifier stored when the authorization URL was built.
    pub code_verifier: &'a PkceVerifier,
    /// Must match the `redirect_uri` used to obtain the code.
    pub redirect_uri: &'a str,
    /// Must match the `client_id` used to obtain the code.
    pub client_id: &'a str,
}

/// The error body returned by the token endpoint on failure.
#[derive(Debug, Deserialize)]
struct OAuthErrorBody {
    error: String,
    error_description: Option<String>,
}

/// Accessor for the `OAuth2` / PKCE endpoints.
#[derive(Debug)]
pub struct OauthApi<'a> {
    client: &'a LichessClient,
}

impl<'a> OauthApi<'a> {
    /// Binds the accessor to a client.
    pub(crate) fn new(client: &'a LichessClient) -> Self {
        Self { client }
    }

    /// Builds the authorization URL plus the `state` and [`PkceVerifier`] to
    /// store for the code exchange.
    ///
    /// This is the user-facing `GET /oauth` endpoint: send the user's browser
    /// to the returned URL rather than requesting it from the client.
    ///
    /// # Errors
    /// Returns [`LichessError::InvalidRequest`] if the configured base URL
    /// cannot be parsed.
    pub fn authorization_url(&self, request: &AuthorizationRequest<'_>) -> Result<Authorization> {
        let verifier = PkceVerifier::generate();
        let state = pkce::random_alphanumeric(STATE_LEN);
        let url = self.build_authorize_url(request, &verifier.code_challenge(), &state)?;
        Ok(Authorization {
            url,
            state,
            verifier,
        })
    }

    /// Assembles the `/oauth` URL with all required query parameters.
    fn build_authorize_url(
        &self,
        request: &AuthorizationRequest<'_>,
        challenge: &str,
        state: &str,
    ) -> Result<Url> {
        let scope = join_scopes(request.scopes);
        let base = self.client.absolute_url(Host::Default, "/oauth");
        let mut params = vec![
            ("response_type", "code"),
            ("client_id", request.client_id),
            ("redirect_uri", request.redirect_uri),
            ("code_challenge_method", "S256"),
            ("code_challenge", challenge),
            ("scope", scope.as_str()),
            ("state", state),
        ];
        if let Some(hint) = request.username_hint {
            params.push(("username", hint));
        }
        Url::parse_with_params(&base, &params)
            .map_err(|err| LichessError::InvalidRequest(format!("invalid base URL: {err}")))
    }

    /// Exchanges an authorization code for an access token.
    ///
    /// `POST /api/token`
    ///
    /// # Errors
    /// Returns [`LichessError::OAuth`] if the token endpoint rejects the
    /// exchange (e.g. `invalid_grant`).
    pub async fn exchange_code(&self, exchange: &CodeExchange<'_>) -> Result<LichessToken> {
        let form = [
            ("grant_type", "authorization_code"),
            ("code", exchange.code),
            ("code_verifier", exchange.code_verifier.as_str()),
            ("redirect_uri", exchange.redirect_uri),
            ("client_id", exchange.client_id),
        ];
        let response = self
            .client
            .request(Method::POST, Host::Default, "/api/token")
            .form(&form)
            .send()
            .await?;
        let status = response.status();
        let bytes = response.bytes().await?;
        if status.is_success() {
            serde_json::from_slice(&bytes).map_err(|err| LichessError::decode("LichessToken", err))
        } else {
            Err(token_failure(status, &bytes))
        }
    }

    /// Revokes the access token the client is currently using.
    ///
    /// `DELETE /api/token`
    pub async fn revoke_token(&self) -> Result<()> {
        let request = self
            .client
            .request(Method::DELETE, Host::Default, "/api/token");
        http::ok(request).await
    }

    /// Tests a set of personal access tokens, returning a map from each token to
    /// its info (`userId`, `scopes`, `expires`) or `null` if invalid.
    ///
    /// `POST /api/token/test`
    pub async fn test_tokens(
        &self,
        tokens: &[&str],
    ) -> Result<std::collections::HashMap<String, serde_json::Value>> {
        let request = self
            .client
            .request(Method::POST, Host::Default, "/api/token/test")
            .text_body(tokens.join(","));
        http::json(request, "token test results").await
    }
}

impl LichessClient {
    /// `OAuth2` API: the PKCE authorization flow and token management.
    #[must_use]
    pub fn oauth(&self) -> OauthApi<'_> {
        OauthApi::new(self)
    }
}

/// Joins scopes into the space-separated form the endpoint expects.
fn join_scopes(scopes: &[Scope]) -> String {
    scopes
        .iter()
        .map(|scope| scope.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Maps a failed token response to a typed error, preferring the OAuth shape.
fn token_failure(status: reqwest::StatusCode, body: &[u8]) -> LichessError {
    match serde_json::from_slice::<OAuthErrorBody>(body) {
        Ok(parsed) => OAuthError::new(&parsed.error, parsed.error_description).into(),
        Err(_) => ApiError::new(status, None, None).into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_scopes_with_spaces() {
        let joined = join_scopes(&[Scope::BoardPlay, Scope::ChallengeWrite]);
        assert_eq!(joined, "board:play challenge:write");
    }

    #[test]
    fn empty_scopes_join_to_empty_string() {
        assert_eq!(join_scopes(&[]), "");
    }

    #[test]
    fn token_failure_prefers_oauth_error() {
        let body = br#"{"error":"invalid_grant","error_description":"bad verifier"}"#;
        let error = token_failure(reqwest::StatusCode::BAD_REQUEST, body);
        match error {
            LichessError::OAuth(oauth) => {
                assert_eq!(oauth.code, crate::error::OAuthErrorCode::InvalidGrant);
            }
            other => panic!("expected OAuth error, got {other:?}"),
        }
    }
}
