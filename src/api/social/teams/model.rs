//! DTOs for the Teams concern.

use serde::{Deserialize, Serialize};

use crate::model::{LichessLightUser, LichessUser};

/// A team.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessTeam {
    /// The team id.
    pub id: String,
    /// The team name.
    pub name: String,
    /// The team description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The team flair.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flair: Option<String>,
    /// The primary leader.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leader: Option<LichessLightUser>,
    /// All team leaders.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leaders: Option<Vec<LichessLightUser>>,
    /// The number of members.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nb_members: Option<u32>,
    /// Whether the team is open to join.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open: Option<bool>,
    /// Whether the authenticated user has joined.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub joined: Option<bool>,
    /// Whether the authenticated user has a pending join request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested: Option<bool>,
}

/// A paginated list of teams.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessTeamPaginator {
    /// The current page number.
    pub current_page: u32,
    /// The maximum results per page.
    pub max_per_page: u32,
    /// The teams on this page.
    pub current_page_results: Vec<LichessTeam>,
    /// The previous page number, if any.
    #[serde(default)]
    pub previous_page: Option<u32>,
    /// The next page number, if any.
    #[serde(default)]
    pub next_page: Option<u32>,
    /// The total number of results.
    pub nb_results: u32,
    /// The total number of pages.
    pub nb_pages: u32,
}

/// A pending join request on a team.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LichessTeamRequest {
    /// The team id.
    pub team_id: String,
    /// The requesting user id.
    pub user_id: String,
    /// When the request was made (Unix milliseconds).
    pub date: i64,
    /// The request message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// A join request together with the requesting user's profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessTeamRequestWithUser {
    /// The request.
    pub request: LichessTeamRequest,
    /// The requesting user.
    pub user: LichessUser,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_team() {
        let json = r#"{"id":"coders","name":"Coders","nbMembers":42,"open":true,
            "leader":{"id":"t","name":"T"}}"#;
        let team: LichessTeam = serde_json::from_str(json).unwrap();
        assert_eq!(team.id, "coders");
        assert_eq!(team.nb_members, Some(42));
        assert_eq!(team.leader.unwrap().name, "T");
    }

    #[test]
    fn parses_paginator_with_null_pages() {
        let json = r#"{"currentPage":1,"maxPerPage":15,"currentPageResults":[],
            "previousPage":null,"nextPage":2,"nbResults":30,"nbPages":2}"#;
        let page: LichessTeamPaginator = serde_json::from_str(json).unwrap();
        assert_eq!(page.previous_page, None);
        assert_eq!(page.next_page, Some(2));
    }

    #[test]
    fn parses_request_with_user() {
        let json = r#"{"request":{"userId":"mary","teamId":"t","date":1,"message":"hi"},
            "user":{"id":"mary","username":"Mary"}}"#;
        let req: LichessTeamRequestWithUser = serde_json::from_str(json).unwrap();
        assert_eq!(req.request.user_id, "mary");
        assert_eq!(req.user.username, "Mary");
    }
}
