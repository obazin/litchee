//! Decoding of newline-delimited JSON (`application/x-ndjson`) responses.
//!
//! Lichess streams many endpoints as NDJSON: one JSON value per line, with
//! empty lines sent periodically for keep-alive. [`ndjson`] turns a response
//! body into a [`Stream`] of deserialized values, buffering across network
//! chunks so values split over chunk boundaries are reassembled.

use futures_core::Stream;
use futures_util::StreamExt;
use serde::de::DeserializeOwned;

use crate::error::{Result, StreamError};

/// Maximum bytes buffered for a single in-progress line. A line longer than this
/// without a newline is treated as a malformed/stalled stream rather than grown
/// without bound. Generous enough never to reject a legitimate Lichess line.
const MAX_LINE_BYTES: usize = 16 * 1024 * 1024;

/// Accumulates response bytes and yields complete `\n`-terminated lines.
///
/// Kept separate from the async machinery so the splitting logic is unit
/// testable in isolation.
#[derive(Debug, Default)]
struct LineSplitter {
    buffer: Vec<u8>,
}

impl LineSplitter {
    /// Appends a chunk of received bytes.
    fn push(&mut self, chunk: &[u8]) {
        self.buffer.extend_from_slice(chunk);
    }

    /// Removes and returns the next complete line (without its `\n`), if one is
    /// fully buffered.
    fn next_line(&mut self) -> Option<Vec<u8>> {
        let newline = self.buffer.iter().position(|byte| *byte == b'\n')?;
        let mut line: Vec<u8> = self.buffer.drain(..=newline).collect();
        line.pop(); // drop the trailing '\n'
        Some(line)
    }

    /// Returns any trailing bytes not terminated by a newline.
    fn finish(self) -> Option<Vec<u8>> {
        (!self.buffer.is_empty()).then_some(self.buffer)
    }

    /// Whether the buffered (still unterminated) line exceeds `max` bytes.
    fn overflowed(&self, max: usize) -> bool {
        self.buffer.len() > max
    }
}

/// Parses one NDJSON line. Blank/whitespace-only lines (keep-alives) yield
/// `Ok(None)`. Pure; unit tested.
fn parse_line<T: DeserializeOwned>(line: &[u8]) -> Result<Option<T>> {
    let trimmed = line.trim_ascii();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let value = serde_json::from_slice(trimmed)
        .map_err(|source| StreamError::decode(String::from_utf8_lossy(trimmed), source))?;
    Ok(Some(value))
}

/// Converts an NDJSON response body into a stream of decoded `T` values.
pub(crate) fn ndjson<T>(response: reqwest::Response) -> impl Stream<Item = Result<T>>
where
    T: DeserializeOwned,
{
    let bytes = response.bytes_stream();
    async_stream::try_stream! {
        let mut splitter = LineSplitter::default();
        futures_util::pin_mut!(bytes);
        while let Some(chunk) = bytes.next().await {
            let chunk = chunk.map_err(StreamError::Transport)?;
            splitter.push(&chunk);
            while let Some(line) = splitter.next_line() {
                if let Some(item) = parse_line(&line)? {
                    yield item;
                }
            }
            if splitter.overflowed(MAX_LINE_BYTES) {
                Err(StreamError::LineTooLong { max: MAX_LINE_BYTES })?;
            }
        }
        // Flatten the trailing-line logic into a single `if let`: let-chains
        // (Rust 2024) aren't usable here because `async_stream` parses this body
        // under its own edition, and a nested `if let` would trip
        // `clippy::collapsible_if`.
        let trailing = splitter.finish().map(|line| parse_line(&line)).transpose()?;
        if let Some(item) = trailing.flatten() {
            yield item;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn splits_lines_across_chunk_boundaries() {
        let mut splitter = LineSplitter::default();
        splitter.push(b"{\"a\":1}\n{\"b\"");
        assert_eq!(splitter.next_line(), Some(b"{\"a\":1}".to_vec()));
        assert_eq!(splitter.next_line(), None);
        splitter.push(b":2}\n");
        assert_eq!(splitter.next_line(), Some(b"{\"b\":2}".to_vec()));
        assert_eq!(splitter.next_line(), None);
        assert_eq!(splitter.finish(), None);
    }

    #[test]
    fn overflowed_only_when_unterminated_buffer_exceeds_max() {
        let mut splitter = LineSplitter::default();
        splitter.push(b"abcde");
        assert!(!splitter.overflowed(5));
        assert!(splitter.overflowed(4));
        // A completed line is drained, so it does not count toward the cap.
        splitter.push(b"\n");
        let _ = splitter.next_line();
        assert!(!splitter.overflowed(0));
    }

    #[test]
    fn finish_returns_unterminated_trailing_line() {
        let mut splitter = LineSplitter::default();
        splitter.push(b"{\"x\":1}");
        assert_eq!(splitter.next_line(), None);
        assert_eq!(splitter.finish(), Some(b"{\"x\":1}".to_vec()));
    }

    #[test]
    fn parse_line_skips_keepalive_blanks() {
        assert!(matches!(parse_line::<Value>(b""), Ok(None)));
        assert!(matches!(parse_line::<Value>(b"   "), Ok(None)));
    }

    #[test]
    fn parse_line_decodes_json() {
        let value: Option<Value> = parse_line(b"{\"ok\":true}").unwrap();
        assert_eq!(value, Some(serde_json::json!({"ok": true})));
    }

    #[test]
    fn parse_line_reports_bad_json_with_the_line() {
        let result = parse_line::<Value>(b"{not json}");
        match result {
            Err(crate::error::LichessError::Stream(StreamError::Decode { line, .. })) => {
                assert_eq!(line, "{not json}");
            }
            other => panic!("expected stream decode error, got {other:?}"),
        }
    }
}
