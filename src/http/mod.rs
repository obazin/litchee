//! The internal request pipeline shared by every endpoint.
//!
//! All requests funnel through [`send`], which applies the **central error
//! mapping**: any non-success response is converted into a typed
//! [`ApiError`](crate::error::ApiError) (status + body message + `Retry-After`).
//! The typed helpers ([`json`], [`text`], [`ok`]) build on it so individual
//! endpoints stay tiny and inherit consistent error handling.

use std::fmt::Display;

use futures_util::stream::BoxStream;
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderName, LAST_MODIFIED, RETRY_AFTER};
use reqwest::{RequestBuilder, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

use crate::error::{ApiError, LichessError, Result};
use crate::retry::RetryPolicy;
use crate::stream::ndjson;

/// The shape of a Lichess error body: `{ "error": "..." }`.
#[derive(Debug, Deserialize)]
struct ErrorBody {
    error: String,
}

/// `Accept` value selecting the newline-delimited JSON representation
/// (`application/x-ndjson`), shared by the streaming game-export endpoints.
pub(crate) const ACCEPT_NDJSON: &str = "application/x-ndjson";

/// `Accept` value selecting the PGN representation (`application/x-chess-pgn`),
/// shared by the game-export endpoints that can return PGN text.
pub(crate) const ACCEPT_PGN: &str = "application/x-chess-pgn";

/// `Accept` value selecting the JSON representation (`application/json`). The
/// single-game and current-game exports default to PGN, so JSON is requested
/// explicitly.
pub(crate) const ACCEPT_JSON: &str = "application/json";

/// `Content-Type` for `application/x-www-form-urlencoded` request bodies.
pub(crate) const CONTENT_TYPE_FORM: &str = "application/x-www-form-urlencoded";

/// Joins already-url-encoded form parts into one body, dropping empty parts.
///
/// `serde_urlencoded` cannot flatten several structs into a single form, so a
/// request that carries a core form plus grouped value-types (conditions, info)
/// serializes each independently and joins the pieces here.
pub(crate) fn join_form(parts: &[String]) -> String {
    parts
        .iter()
        .filter(|part| !part.is_empty())
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join("&")
}

/// Characters left literal in a path segment: the RFC 3986 unreserved set
/// (`A-Z a-z 0-9 - . _ ~`). Everything else — including `/ ? # %` — is encoded.
const PATH_SEGMENT: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

/// Percent-encodes a single URL path segment.
///
/// Caller-supplied path parameters (usernames, ids, slugs) must go through this
/// so a value containing `/`, `?`, `#`, or `..` cannot reshape the request path.
/// Returns a [`Display`] adapter so it can be written straight into a `format!`
/// without an intermediate `String` allocation.
pub(crate) fn segment(value: &str) -> impl Display + '_ {
    utf8_percent_encode(value, PATH_SEGMENT)
}

/// A request that carries the client's [`RetryPolicy`] so [`send`] can retry
/// rate-limited (`429`) responses. Mirrors the small slice of
/// `reqwest::RequestBuilder` the endpoints actually use.
#[derive(Debug)]
pub(crate) struct ApiRequest {
    builder: RequestBuilder,
    retry: RetryPolicy,
}

impl ApiRequest {
    /// Wraps a builder with the policy to apply when it is sent.
    pub(crate) fn new(builder: RequestBuilder, retry: RetryPolicy) -> Self {
        Self { builder, retry }
    }

    /// Adds a request header (all call sites use static values).
    pub(crate) fn header(mut self, name: HeaderName, value: &'static str) -> Self {
        self.builder = self.builder.header(name, value);
        self
    }

    /// Sets the URL query string from a serializable value.
    pub(crate) fn query<T: Serialize + ?Sized>(mut self, query: &T) -> Self {
        self.builder = self.builder.query(query);
        self
    }

    /// Sets a URL-encoded form body.
    pub(crate) fn form<T: Serialize + ?Sized>(mut self, form: &T) -> Self {
        self.builder = self.builder.form(form);
        self
    }

    /// Sets an `x-www-form-urlencoded` body from a core form plus a grouped
    /// value-type, serializing each independently and joining the non-empty
    /// parts (with the content-type set in one place).
    ///
    /// `serde_urlencoded` cannot flatten the grouped value-types (conditions,
    /// info) into the core struct, so they are encoded separately here. Both
    /// arguments are flat structs, so serialization cannot fail.
    pub(crate) fn form_parts<A, B>(self, core: &A, extra: &B) -> Self
    where
        A: Serialize + ?Sized,
        B: Serialize + ?Sized,
    {
        let core = serde_urlencoded::to_string(core).unwrap_or_default();
        let extra = serde_urlencoded::to_string(extra).unwrap_or_default();
        self.header(CONTENT_TYPE, CONTENT_TYPE_FORM)
            .body(join_form(&[core, extra]))
    }

    /// Sets a JSON body.
    pub(crate) fn json<T: Serialize + ?Sized>(mut self, json: &T) -> Self {
        self.builder = self.builder.json(json);
        self
    }

    /// Sets a raw body.
    pub(crate) fn body(mut self, body: impl Into<reqwest::Body>) -> Self {
        self.builder = self.builder.body(body);
        self
    }

    /// Sends the request, retrying `429` responses per the policy. Returns the
    /// final response (success or not); the caller classifies the status.
    pub(crate) async fn send(self) -> reqwest::Result<Response> {
        let Self { mut builder, retry } = self;
        let mut attempt = 0;
        loop {
            let next = (attempt < retry.max_retries())
                .then(|| builder.try_clone())
                .flatten();
            let response = builder.send().await?;
            let Some(retry_builder) = next.filter(|_| is_rate_limited(&response)) else {
                return Ok(response);
            };
            sleep(retry.delay(attempt, response.headers())).await;
            builder = retry_builder;
            attempt += 1;
        }
    }

    /// Builds the underlying `reqwest::Request` (for inspection in tests).
    #[cfg(test)]
    pub(crate) fn build(self) -> reqwest::Result<reqwest::Request> {
        self.builder.build()
    }
}

/// Whether a response is a rate-limit rejection that is safe to retry.
fn is_rate_limited(response: &Response) -> bool {
    response.status() == StatusCode::TOO_MANY_REQUESTS
}

/// Sends a request, mapping any non-success status to a typed error.
pub(crate) async fn send(request: ApiRequest) -> Result<Response> {
    let response = request.send().await?;
    if response.status().is_success() {
        Ok(response)
    } else {
        Err(api_error(response).await)
    }
}

/// Consumes a failed response into a typed [`ApiError`].
async fn api_error(response: Response) -> LichessError {
    let status = response.status();
    let retry_after = retry_after_secs(response.headers());
    let body = response.text().await.unwrap_or_default();
    ApiError::new(status, error_message(&body), retry_after).into()
}

/// Extracts the `error` field from a Lichess error body, if present. Pure.
fn error_message(body: &str) -> Option<String> {
    serde_json::from_str::<ErrorBody>(body)
        .ok()
        .map(|parsed| parsed.error)
}

/// Parses the `Retry-After` header as a whole number of seconds. Pure.
fn retry_after_secs(headers: &HeaderMap) -> Option<u64> {
    headers.get(RETRY_AFTER)?.to_str().ok()?.trim().parse().ok()
}

/// Reads the `Last-Modified` response header as an owned string, if present. Pure.
pub(crate) fn last_modified(headers: &HeaderMap) -> Option<String> {
    Some(headers.get(LAST_MODIFIED)?.to_str().ok()?.to_owned())
}

/// Sends a request and deserializes a JSON body into `T`.
///
/// `context` names what is being decoded, for clearer error messages.
pub(crate) async fn json<T: DeserializeOwned>(request: ApiRequest, context: &str) -> Result<T> {
    let response = send(request).await?;
    let bytes = response.bytes().await?;
    serde_json::from_slice(&bytes).map_err(|source| LichessError::decode(context, source))
}

/// Sends a request and returns the body as text (for PGN / `text/plain`).
pub(crate) async fn text(request: ApiRequest) -> Result<String> {
    let response = send(request).await?;
    Ok(response.text().await?)
}

/// Sends a request that returns `{ "ok": true }` or `204 No Content`, where only
/// success matters. The body is discarded once the status is validated.
pub(crate) async fn ok(request: ApiRequest) -> Result<()> {
    let response = send(request).await?;
    drop(response.bytes().await?);
    Ok(())
}

/// Sends a request and streams its NDJSON body as decoded `T` values.
///
/// The initial request (and thus any HTTP error) is resolved before the stream
/// is returned; per-line decode failures surface as items in the stream. The
/// stream is boxed (and thus `Unpin`) so callers can consume it directly with
/// [`StreamExt::next`](futures_util::StreamExt::next). `max_line_bytes` bounds a
/// single buffered NDJSON line (see [`ndjson`](crate::stream::ndjson)).
pub(crate) async fn stream<T: DeserializeOwned + Send + 'static>(
    request: ApiRequest,
    max_line_bytes: usize,
) -> Result<BoxStream<'static, Result<T>>> {
    let response = send(request).await?;
    Ok(Box::pin(ndjson(response, max_line_bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::{HeaderValue, RETRY_AFTER};

    #[test]
    fn extracts_error_message_from_body() {
        assert_eq!(
            error_message(r#"{"error":"Not found."}"#),
            Some("Not found.".to_owned())
        );
    }

    #[test]
    fn ignores_non_error_bodies() {
        assert_eq!(error_message(r#"{"ok":true}"#), None);
        assert_eq!(error_message("plain text"), None);
    }

    #[test]
    fn join_form_drops_empty_parts() {
        assert_eq!(join_form(&[]), "");
        assert_eq!(join_form(&["a=1".to_owned(), String::new()]), "a=1");
        assert_eq!(join_form(&[String::new(), "b=2".to_owned()]), "b=2");
        assert_eq!(join_form(&["a=1".to_owned(), "b=2".to_owned()]), "a=1&b=2");
        assert_eq!(join_form(&[String::new(), String::new()]), "");
    }

    #[test]
    fn parses_retry_after_seconds() {
        let mut headers = HeaderMap::new();
        headers.insert(RETRY_AFTER, HeaderValue::from_static("60"));
        assert_eq!(retry_after_secs(&headers), Some(60));
    }

    #[test]
    fn missing_retry_after_is_none() {
        assert_eq!(retry_after_secs(&HeaderMap::new()), None);
    }

    #[test]
    fn reads_last_modified_header() {
        let mut headers = HeaderMap::new();
        headers.insert(
            LAST_MODIFIED,
            HeaderValue::from_static("Tue, 25 Apr 2023 13:23:09 GMT"),
        );
        assert_eq!(
            last_modified(&headers).as_deref(),
            Some("Tue, 25 Apr 2023 13:23:09 GMT")
        );
        assert_eq!(last_modified(&HeaderMap::new()), None);
    }

    #[test]
    fn segment_encodes_path_breaking_characters() {
        assert_eq!(segment("../a").to_string(), "..%2Fa");
        assert_eq!(segment("a/b").to_string(), "a%2Fb");
        assert_eq!(segment("a?b#c").to_string(), "a%3Fb%23c");
        assert_eq!(segment("a b%c").to_string(), "a%20b%25c");
    }

    #[test]
    fn segment_leaves_unreserved_characters_intact() {
        assert_eq!(segment("normal-id_1.x~").to_string(), "normal-id_1.x~");
        assert_eq!(segment("Lichess123").to_string(), "Lichess123");
    }
}
