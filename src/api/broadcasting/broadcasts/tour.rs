//! Tournament creation/edit builder and its form types.

use reqwest::Method;
use serde::Serialize;

use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;

use super::LichessBroadcast;

/// Form body for creating/editing a broadcast tournament (flat, non-`info`
/// fields).
#[derive(Debug, Default, Serialize)]
struct TourForm<'a> {
    name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    visibility: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    markdown: Option<&'a str>,
    #[serde(rename = "showScores", skip_serializing_if = "Option::is_none")]
    show_scores: Option<bool>,
    #[serde(rename = "showRatingDiffs", skip_serializing_if = "Option::is_none")]
    show_rating_diffs: Option<bool>,
    #[serde(rename = "teamTable", skip_serializing_if = "Option::is_none")]
    team_table: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    players: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    teams: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tier: Option<u8>,
}

/// Display information for a broadcast tournament, serialized as `info.*` keys.
#[derive(Debug, Clone, Default, Serialize)]
pub struct BroadcastTourInfo<'a> {
    #[serde(rename = "info.format", skip_serializing_if = "Option::is_none")]
    format: Option<&'a str>,
    #[serde(rename = "info.tc", skip_serializing_if = "Option::is_none")]
    tc: Option<&'a str>,
    #[serde(rename = "info.fideTC", skip_serializing_if = "Option::is_none")]
    fide_tc: Option<&'a str>,
    #[serde(rename = "info.timeZone", skip_serializing_if = "Option::is_none")]
    time_zone: Option<&'a str>,
    #[serde(rename = "info.location", skip_serializing_if = "Option::is_none")]
    location: Option<&'a str>,
    #[serde(rename = "info.players", skip_serializing_if = "Option::is_none")]
    players: Option<&'a str>,
    #[serde(rename = "info.website", skip_serializing_if = "Option::is_none")]
    website: Option<&'a str>,
    #[serde(rename = "info.standings", skip_serializing_if = "Option::is_none")]
    standings: Option<&'a str>,
    #[serde(rename = "info.regulations", skip_serializing_if = "Option::is_none")]
    regulations: Option<&'a str>,
}

impl<'a> BroadcastTourInfo<'a> {
    /// Tournament format, e.g. `"8-player round-robin"`.
    #[must_use]
    pub fn format(mut self, format: &'a str) -> Self {
        self.format = Some(format);
        self
    }

    /// Time control, e.g. `"Classical"` or `"Rapid & Blitz"`.
    #[must_use]
    pub fn tc(mut self, tc: &'a str) -> Self {
        self.tc = Some(tc);
        self
    }

    /// FIDE rating category (`standard`, `rapid`, or `blitz`).
    #[must_use]
    pub fn fide_tc(mut self, fide_tc: &'a str) -> Self {
        self.fide_tc = Some(fide_tc);
        self
    }

    /// Timezone identifier, e.g. `America/New_York`.
    #[must_use]
    pub fn time_zone(mut self, time_zone: &'a str) -> Self {
        self.time_zone = Some(time_zone);
        self
    }

    /// Tournament location.
    #[must_use]
    pub fn location(mut self, location: &'a str) -> Self {
        self.location = Some(location);
        self
    }

    /// Up to four of the best participating players.
    #[must_use]
    pub fn players(mut self, players: &'a str) -> Self {
        self.players = Some(players);
        self
    }

    /// Official website URL.
    #[must_use]
    pub fn website(mut self, website: &'a str) -> Self {
        self.website = Some(website);
        self
    }

    /// Official standings website URL.
    #[must_use]
    pub fn standings(mut self, standings: &'a str) -> Self {
        self.standings = Some(standings);
        self
    }

    /// External URL to the tournament regulations.
    #[must_use]
    pub fn regulations(mut self, regulations: &'a str) -> Self {
        self.regulations = Some(regulations);
        self
    }
}

/// Grouping configuration for a broadcast tournament, serialized as the nested
/// `grouping.*` form keys (`grouping.info.name`, `grouping.info.tours`, and
/// `grouping.scoreGroups[i]`).
#[derive(Debug, Clone, Default)]
pub struct BroadcastGrouping<'a> {
    name: Option<&'a str>,
    tours: Option<&'a str>,
    score_groups: Vec<&'a str>,
}

impl<'a> BroadcastGrouping<'a> {
    /// Sets the group name.
    #[must_use]
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self
    }

    /// Sets the linebreak-separated tournament ids to group together.
    #[must_use]
    pub fn tours(mut self, tours: &'a str) -> Self {
        self.tours = Some(tours);
        self
    }

    /// Adds a score group (comma-separated tournament ids). Call repeatedly for
    /// several groups; ids must be a subset of [`tours`](Self::tours).
    #[must_use]
    pub fn score_group(mut self, tours: &'a str) -> Self {
        self.score_groups.push(tours);
        self
    }

    /// Appends the `grouping.*` form pairs; adds nothing when unset.
    fn append_pairs(&self, out: &mut Vec<(String, String)>) {
        if let Some(name) = self.name {
            out.push(("grouping.info.name".to_owned(), name.to_owned()));
        }
        if let Some(tours) = self.tours {
            out.push(("grouping.info.tours".to_owned(), tours.to_owned()));
        }
        for (index, group) in self.score_groups.iter().enumerate() {
            out.push((
                format!("grouping.scoreGroups[{index}]"),
                (*group).to_owned(),
            ));
        }
    }
}

/// Builder for creating or editing a broadcast tournament.
#[derive(Debug)]
pub struct TourRequest<'a> {
    client: &'a LichessClient,
    edit_id: Option<&'a str>,
    form: TourForm<'a>,
    info: BroadcastTourInfo<'a>,
    tiebreaks: Vec<&'a str>,
    grouping: BroadcastGrouping<'a>,
}

impl<'a> TourRequest<'a> {
    /// Creates the request builder.
    pub(crate) fn new(client: &'a LichessClient, edit_id: Option<&'a str>, name: &'a str) -> Self {
        Self {
            client,
            edit_id,
            form: TourForm {
                name,
                ..Default::default()
            },
            info: BroadcastTourInfo::default(),
            tiebreaks: Vec::new(),
            grouping: BroadcastGrouping::default(),
        }
    }

    /// Sets the structured display information.
    #[must_use]
    pub fn info(mut self, info: BroadcastTourInfo<'a>) -> Self {
        self.info = info;
        self
    }

    /// Sets the visibility (`public`, `unlisted`, or `private`).
    #[must_use]
    pub fn visibility(mut self, visibility: &'a str) -> Self {
        self.form.visibility = Some(visibility);
        self
    }

    /// Sets a long Markdown description.
    #[must_use]
    pub fn markdown(mut self, markdown: &'a str) -> Self {
        self.form.markdown = Some(markdown);
        self
    }

    /// Sets whether to show player scores.
    #[must_use]
    pub fn show_scores(mut self, value: bool) -> Self {
        self.form.show_scores = Some(value);
        self
    }

    /// Sets whether to show rating differences.
    #[must_use]
    pub fn show_rating_diffs(mut self, value: bool) -> Self {
        self.form.show_rating_diffs = Some(value);
        self
    }

    /// Sets whether to display a team table.
    #[must_use]
    pub fn team_table(mut self, value: bool) -> Self {
        self.form.team_table = Some(value);
        self
    }

    /// Sets player tags / overrides (one line per player).
    #[must_use]
    pub fn players(mut self, players: &'a str) -> Self {
        self.form.players = Some(players);
        self
    }

    /// Assigns players to teams (one line per player).
    #[must_use]
    pub fn teams(mut self, teams: &'a str) -> Self {
        self.form.teams = Some(teams);
        self
    }

    /// Sets the broadcast tier (`3`, `4`, or `5`).
    #[must_use]
    pub fn tier(mut self, tier: u8) -> Self {
        self.form.tier = Some(tier);
        self
    }

    /// Sets the tiebreak ordering (extended tiebreak codes, e.g. `BH`, `AOB`;
    /// up to 5).
    #[must_use]
    pub fn tiebreaks(mut self, tiebreaks: &[&'a str]) -> Self {
        self.tiebreaks = tiebreaks.to_vec();
        self
    }

    /// Sets the grouping configuration (group this broadcast with others).
    #[must_use]
    pub fn grouping(mut self, grouping: BroadcastGrouping<'a>) -> Self {
        self.grouping = grouping;
        self
    }

    /// Creates or updates the tournament.
    pub async fn send(self) -> Result<LichessBroadcast> {
        let path = match self.edit_id {
            Some(id) => format!("/broadcast/{}/edit", http::segment(id)),
            None => "/broadcast/new".to_owned(),
        };
        let extra_pairs = self.extra_pairs();
        let core = serde_urlencoded::to_string(&self.form).unwrap_or_default();
        let info = serde_urlencoded::to_string(&self.info).unwrap_or_default();
        let extra = serde_urlencoded::to_string(&extra_pairs).unwrap_or_default();
        let request = self
            .client
            .request(Method::POST, Host::Default, &path)
            .form_body(http::join_form(&[core, info, extra]));
        http::json(request, "LichessBroadcast").await
    }

    /// Builds the array/nested `tiebreaks[i]` + `grouping.*` form pairs, which
    /// `serde_urlencoded` cannot express as struct fields.
    fn extra_pairs(&self) -> Vec<(String, String)> {
        let mut pairs = Vec::new();
        for (index, code) in self.tiebreaks.iter().enumerate() {
            pairs.push((format!("tiebreaks[{index}]"), (*code).to_owned()));
        }
        self.grouping.append_pairs(&mut pairs);
        pairs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tour_info_serializes_to_dotted_keys() {
        let query = serde_urlencoded::to_string(
            BroadcastTourInfo::default()
                .format("8-player RR")
                .fide_tc("standard"),
        )
        .unwrap();
        assert!(query.contains("info.format=8-player+RR"));
        assert!(query.contains("info.fideTC=standard"));
    }

    #[test]
    fn empty_tour_info_serializes_to_nothing() {
        assert_eq!(
            serde_urlencoded::to_string(BroadcastTourInfo::default()).unwrap(),
            ""
        );
    }

    #[test]
    fn grouping_encodes_nested_and_indexed_keys() {
        let mut pairs = vec![("tiebreaks[0]".to_owned(), "BH".to_owned())];
        BroadcastGrouping::default()
            .name("Open")
            .tours("a,b")
            .score_group("a,b")
            .append_pairs(&mut pairs);
        let encoded = serde_urlencoded::to_string(&pairs).unwrap();
        assert_eq!(
            encoded,
            "tiebreaks%5B0%5D=BH&grouping.info.name=Open\
             &grouping.info.tours=a%2Cb&grouping.scoreGroups%5B0%5D=a%2Cb"
        );
    }
}
