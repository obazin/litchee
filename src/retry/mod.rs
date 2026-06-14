//! Opt-in retry policy for rate-limited requests.
//!
//! Lichess returns `429 Too Many Requests` (with a `Retry-After` header) when a
//! client exceeds a rate limit. A `429` means the request was *not* processed,
//! so retrying it after the advised delay is safe for any HTTP method. This
//! policy retries only that case; transient `5xx` and transport errors are left
//! to the caller, since retrying a non-idempotent request that may have been
//! processed is unsafe.

use std::time::Duration;

use reqwest::header::{HeaderMap, RETRY_AFTER};

/// Default base delay for exponential backoff when no `Retry-After` is given.
const DEFAULT_BASE_DELAY: Duration = Duration::from_secs(1);
/// Default ceiling on any single retry wait.
const DEFAULT_MAX_DELAY: Duration = Duration::from_mins(1);

/// Controls automatic retries of rate-limited (`429`) requests.
///
/// Retries are **opt-in**: the default policy performs none, preserving the
/// fail-fast behaviour. Enable them on the client builder:
///
/// ```
/// use std::time::Duration;
/// use litchee::RetryPolicy;
///
/// let policy = RetryPolicy::new(3).with_max_delay(Duration::from_secs(30));
/// let client = litchee::LichessClient::builder().retry_policy(policy).build();
/// ```
///
/// On a `429`, the wait is the response's `Retry-After` value when present,
/// otherwise exponential backoff (`base_delay * 2^attempt`); either way it is
/// clamped to `max_delay`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryPolicy {
    max_retries: u32,
    base_delay: Duration,
    max_delay: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 0,
            base_delay: DEFAULT_BASE_DELAY,
            max_delay: DEFAULT_MAX_DELAY,
        }
    }
}

impl RetryPolicy {
    /// A policy that retries a rate-limited request up to `max_retries` times.
    #[must_use]
    pub fn new(max_retries: u32) -> Self {
        Self {
            max_retries,
            ..Self::default()
        }
    }

    /// Sets the base delay for exponential backoff (default 1s).
    #[must_use]
    pub fn with_base_delay(mut self, base_delay: Duration) -> Self {
        self.base_delay = base_delay;
        self
    }

    /// Sets the ceiling on any single retry wait (default 60s).
    #[must_use]
    pub fn with_max_delay(mut self, max_delay: Duration) -> Self {
        self.max_delay = max_delay;
        self
    }

    /// The maximum number of retries this policy permits.
    pub(crate) fn max_retries(&self) -> u32 {
        self.max_retries
    }

    /// The delay to wait before retry `attempt` (0-based), preferring the
    /// response's `Retry-After`, else exponential backoff; clamped to `max_delay`.
    pub(crate) fn delay(&self, attempt: u32, headers: &HeaderMap) -> Duration {
        retry_after(headers)
            .unwrap_or_else(|| self.backoff(attempt))
            .min(self.max_delay)
    }

    /// `base_delay * 2^attempt`, saturating rather than overflowing.
    fn backoff(&self, attempt: u32) -> Duration {
        let factor = 2u32.saturating_pow(attempt);
        self.base_delay.saturating_mul(factor)
    }
}

/// Parses the `Retry-After` header as a whole number of seconds. Pure.
///
/// Only the delta-seconds form (which Lichess sends) is recognised; an
/// HTTP-date value returns `None` and the caller falls back to backoff.
fn retry_after(headers: &HeaderMap) -> Option<Duration> {
    let secs: u64 = headers
        .get(RETRY_AFTER)?
        .to_str()
        .ok()?
        .trim()
        .parse()
        .ok()?;
    Some(Duration::from_secs(secs))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers_with_retry_after(value: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(RETRY_AFTER, value.parse().unwrap());
        headers
    }

    #[test]
    fn default_policy_does_not_retry() {
        assert_eq!(RetryPolicy::default().max_retries(), 0);
    }

    #[test]
    fn prefers_retry_after_when_present() {
        let policy = RetryPolicy::new(3);
        let delay = policy.delay(0, &headers_with_retry_after("5"));
        assert_eq!(delay, Duration::from_secs(5));
    }

    #[test]
    fn falls_back_to_exponential_backoff() {
        let policy = RetryPolicy::new(3).with_base_delay(Duration::from_secs(2));
        let empty = HeaderMap::new();
        assert_eq!(policy.delay(0, &empty), Duration::from_secs(2));
        assert_eq!(policy.delay(1, &empty), Duration::from_secs(4));
        assert_eq!(policy.delay(2, &empty), Duration::from_secs(8));
    }

    #[test]
    fn clamps_to_max_delay() {
        let policy = RetryPolicy::new(10)
            .with_base_delay(Duration::from_secs(10))
            .with_max_delay(Duration::from_secs(30));
        // Backoff would be 10 * 2^5 = 320s; clamped to 30s.
        assert_eq!(policy.delay(5, &HeaderMap::new()), Duration::from_secs(30));
        // A large Retry-After is clamped too.
        assert_eq!(
            policy.delay(0, &headers_with_retry_after("9999")),
            Duration::from_secs(30)
        );
    }

    #[test]
    fn backoff_saturates_without_overflow() {
        let policy = RetryPolicy::new(99).with_max_delay(Duration::from_mins(1));
        assert_eq!(
            policy.delay(u32::MAX, &HeaderMap::new()),
            Duration::from_mins(1)
        );
    }
}
