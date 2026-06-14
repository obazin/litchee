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
use reqwest::header::{HeaderMap, LAST_MODIFIED, RETRY_AFTER};
use reqwest::{RequestBuilder, Response};
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::error::{ApiError, LichessError, Result};
use crate::stream::ndjson;

/// The shape of a Lichess error body: `{ "error": "..." }`.
#[derive(Debug, Deserialize)]
struct ErrorBody {
    error: String,
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

/// Sends a request, mapping any non-success status to a typed error.
pub(crate) async fn send(builder: RequestBuilder) -> Result<Response> {
    let response = builder.send().await?;
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
pub(crate) async fn json<T: DeserializeOwned>(builder: RequestBuilder, context: &str) -> Result<T> {
    let response = send(builder).await?;
    let bytes = response.bytes().await?;
    serde_json::from_slice(&bytes).map_err(|source| LichessError::decode(context, source))
}

/// Sends a request and returns the body as text (for PGN / `text/plain`).
pub(crate) async fn text(builder: RequestBuilder) -> Result<String> {
    let response = send(builder).await?;
    Ok(response.text().await?)
}

/// Sends a request that returns `{ "ok": true }` or `204 No Content`, where only
/// success matters. The body is discarded once the status is validated.
pub(crate) async fn ok(builder: RequestBuilder) -> Result<()> {
    let response = send(builder).await?;
    drop(response.bytes().await?);
    Ok(())
}

/// Sends a request and streams its NDJSON body as decoded `T` values.
///
/// The initial request (and thus any HTTP error) is resolved before the stream
/// is returned; per-line decode failures surface as items in the stream. The
/// stream is boxed (and thus `Unpin`) so callers can consume it directly with
/// [`StreamExt::next`](futures_util::StreamExt::next).
pub(crate) async fn stream<T: DeserializeOwned + Send + 'static>(
    builder: RequestBuilder,
) -> Result<BoxStream<'static, Result<T>>> {
    let response = send(builder).await?;
    Ok(Box::pin(ndjson(response)))
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
