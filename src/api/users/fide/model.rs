//! DTOs for the FIDE concern.

use serde::{Deserialize, Serialize};

use crate::model::LichessTitle;

/// FIDE's mandatory binary gender field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LichessFideGender {
    /// Male.
    #[serde(rename = "M")]
    Male,
    /// Female.
    #[serde(rename = "F")]
    Female,
}

/// Photo URLs for a FIDE player.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessFidePhoto {
    /// URL of a small (100x100) thumbnail.
    pub small: String,
    /// URL of a medium (500x500) version.
    pub medium: String,
    /// Attribution to display next to the photo, if set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credit: Option<String>,
}

/// A FIDE-rated player.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessFidePlayer {
    /// The FIDE player id.
    pub id: u32,
    /// Full name.
    pub name: String,
    /// Title, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<LichessTitle>,
    /// Federation code.
    pub federation: String,
    /// Birth year, if known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub year: Option<u32>,
    /// Inactivity marker.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inactive: Option<u32>,
    /// Standard rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub standard: Option<u32>,
    /// Rapid rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rapid: Option<u32>,
    /// Blitz rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blitz: Option<u32>,
    /// Gender.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gender: Option<LichessFideGender>,
    /// Photo URLs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub photo: Option<LichessFidePhoto>,
}

/// Encoded FIDE rating histories.
///
/// Each entry encodes a year, month, and rating in one number, e.g.
/// `2015081568` means `August 2015: 1568`. See
/// [`decode_point`](LichessFidePlayerRatings::decode_point).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessFidePlayerRatings {
    /// Standard rating history.
    pub standard: Vec<u64>,
    /// Rapid rating history.
    pub rapid: Vec<u64>,
    /// Blitz rating history.
    pub blitz: Vec<u64>,
}

impl LichessFidePlayerRatings {
    /// Decodes one encoded point into `(year, month, rating)`.
    #[must_use]
    pub fn decode_point(point: u64) -> (u64, u64, u64) {
        let rating = point % 10_000;
        let date = point / 10_000;
        (date / 100, date % 100, rating)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_player_with_null_year() {
        let json = r#"{"id":35009192,"name":"Erigaisi Arjun","federation":"IND","year":null}"#;
        let player: LichessFidePlayer = serde_json::from_str(json).unwrap();
        assert_eq!(player.id, 35_009_192);
        assert_eq!(player.year, None);
    }

    #[test]
    fn parses_gender_codes() {
        let json = r#"{"id":1,"name":"X","federation":"FRA","gender":"F"}"#;
        let player: LichessFidePlayer = serde_json::from_str(json).unwrap();
        assert_eq!(player.gender, Some(LichessFideGender::Female));
    }

    #[test]
    fn decodes_a_rating_point() {
        assert_eq!(
            LichessFidePlayerRatings::decode_point(2_015_081_568),
            (2015, 8, 1568)
        );
    }
}
