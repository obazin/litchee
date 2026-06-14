//! A wrapper that keeps a sensitive value out of `Debug` output.

use std::fmt;

use serde::{Deserialize, Serialize};

/// The placeholder rendered in place of a redacted value. The single source of
/// truth for the redaction marker used across the crate.
pub(crate) const REDACTED: &str = "<redacted>";

/// A value that must not appear in logs.
///
/// `Secret` is transparent for serialization (it serializes and deserializes as
/// the inner value), implements the usual comparison and clone traits, but its
/// [`Debug`] output is always `<redacted>`. This makes redaction *secure by
/// construction*: a field typed `Secret<T>` cannot leak through a derived
/// `Debug`, and a new secret field is safe the moment it is declared.
///
/// Read the inner value explicitly with [`expose`](Self::expose).
#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Secret<T = String>(T);

impl<T> Secret<T> {
    /// Wraps a sensitive value.
    pub const fn new(value: T) -> Self {
        Self(value)
    }

    /// Returns a reference to the wrapped value, making the access explicit.
    pub const fn expose(&self) -> &T {
        &self.0
    }

    /// Consumes the wrapper and returns the inner value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> From<T> for Secret<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(REDACTED)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_is_always_redacted() {
        let secret = Secret::new("lip_supersecret".to_owned());
        assert_eq!(format!("{secret:?}"), REDACTED);
        assert!(!format!("{secret:?}").contains("lip_supersecret"));
    }

    #[test]
    fn expose_returns_the_inner_value() {
        let secret = Secret::new("abc");
        assert_eq!(secret.expose(), &"abc");
        assert_eq!(Secret::from("abc").into_inner(), "abc");
    }

    #[test]
    fn serializes_transparently_as_the_inner_value() {
        let secret = Secret::new("abc".to_owned());
        assert_eq!(serde_json::to_string(&secret).unwrap(), "\"abc\"");
        let back: Secret<String> = serde_json::from_str("\"abc\"").unwrap();
        assert_eq!(back.expose(), "abc");
    }
}
