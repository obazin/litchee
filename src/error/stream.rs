//! Errors raised while decoding a streamed (NDJSON) response.

/// A failure encountered while consuming a streaming endpoint.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum StreamError {
    /// The underlying HTTP body stream failed (connection reset, timeout, …).
    #[error("stream transport error")]
    Transport(#[source] reqwest::Error),

    /// A single NDJSON line could not be deserialized into the target type.
    #[error("failed to decode streamed JSON line")]
    Decode {
        /// The raw line that failed to parse.
        line: String,
        /// The underlying deserialization error.
        #[source]
        source: serde_json::Error,
    },

    /// A single line exceeded the configured buffer limit without a terminating
    /// newline. This is a deliberate guard against unbounded memory growth on a
    /// malformed or stalled stream — not a parse error; the limit is tunable via
    /// [`LichessClientBuilder::max_line_bytes`](crate::LichessClientBuilder::max_line_bytes).
    #[error("streamed line exceeded the {max}-byte limit without a newline")]
    LineTooLong {
        /// The configured maximum line length, in bytes.
        max: usize,
    },
}

impl StreamError {
    /// Builds a [`StreamError::Decode`] for a line that failed to parse.
    pub(crate) fn decode(line: impl Into<String>, source: serde_json::Error) -> Self {
        Self::Decode {
            line: line.into(),
            source,
        }
    }
}
