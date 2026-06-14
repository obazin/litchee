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

    /// A line exceeded the maximum buffered size without a terminating newline,
    /// indicating a stalled or malformed stream; the buffer is capped to avoid
    /// unbounded memory growth.
    #[error("streamed line exceeded the {max}-byte limit without a newline")]
    LineTooLong {
        /// The maximum line length, in bytes.
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
