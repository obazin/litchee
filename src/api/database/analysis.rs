//! The Analysis API: cloud evaluations of positions.
//!
//! Reached through [`LichessClient::analysis`].

use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::client::LichessClient;
use crate::config::Host;
use crate::error::Result;
use crate::http;
use crate::model::LichessVariantKey;

/// Accessor for the Analysis API.
#[derive(Debug)]
pub struct AnalysisApi<'a> {
    client: &'a LichessClient,
}

impl<'a> AnalysisApi<'a> {
    /// Binds the accessor to a client.
    pub(crate) fn new(client: &'a LichessClient) -> Self {
        Self { client }
    }

    /// Starts a request for the cloud evaluation of a position.
    ///
    /// Call [`send`](CloudEvalRequest::send) to execute it.
    /// `GET /api/cloud-eval`
    #[must_use]
    pub fn cloud_eval(&self, fen: &'a str) -> CloudEvalRequest<'a> {
        CloudEvalRequest {
            client: self.client,
            fen,
            multi_pv: None,
            variant: None,
        }
    }
}

/// Builder for a cloud-evaluation request.
#[derive(Debug)]
pub struct CloudEvalRequest<'a> {
    client: &'a LichessClient,
    fen: &'a str,
    multi_pv: Option<u8>,
    variant: Option<LichessVariantKey>,
}

/// Query parameters for `GET /api/cloud-eval`.
#[derive(Debug, Serialize)]
struct CloudEvalQuery<'a> {
    fen: &'a str,
    #[serde(rename = "multiPv", skip_serializing_if = "Option::is_none")]
    multi_pv: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    variant: Option<LichessVariantKey>,
}

impl CloudEvalRequest<'_> {
    /// Sets the number of variations to return (default 1).
    #[must_use]
    pub fn multi_pv(mut self, count: u8) -> Self {
        self.multi_pv = Some(count);
        self
    }

    /// Sets the variant of the position.
    #[must_use]
    pub fn variant(mut self, variant: LichessVariantKey) -> Self {
        self.variant = Some(variant);
        self
    }

    /// Executes the request.
    ///
    /// # Errors
    /// Returns [`crate::error::ApiErrorKind::NotFound`] if the position is not
    /// in the cloud-evaluation database.
    pub async fn send(self) -> Result<LichessCloudEval> {
        let query = CloudEvalQuery {
            fen: self.fen,
            multi_pv: self.multi_pv,
            variant: self.variant,
        };
        let request = self
            .client
            .request(Method::GET, Host::Default, "/api/cloud-eval")
            .query(&query);
        http::json(request, "LichessCloudEval").await
    }
}

impl LichessClient {
    /// Analysis API: cloud evaluations of positions.
    #[must_use]
    pub fn analysis(&self) -> AnalysisApi<'_> {
        AnalysisApi::new(self)
    }
}

/// A cached cloud evaluation of a position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessCloudEval {
    /// The evaluated position (X-FEN).
    pub fen: String,
    /// Nodes searched, in thousands.
    pub knodes: u64,
    /// Search depth.
    pub depth: u32,
    /// The principal variations (up to 5).
    pub pvs: Vec<LichessCloudEvalPv>,
}

/// One principal variation from a [`LichessCloudEval`].
///
/// Exactly one of [`cp`](Self::cp) or [`mate`](Self::mate) is present.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LichessCloudEvalPv {
    /// The variation in UCI notation.
    pub moves: String,
    /// Evaluation in centipawns, from White's point of view.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cp: Option<i32>,
    /// Moves to mate, from White's point of view.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mate: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cloud_eval_with_cp_and_mate_variations() {
        let json = r#"{"fen":"r1bqkbnr","knodes":106325,"depth":29,
            "pvs":[{"moves":"d1e2 d8e7","cp":41},{"moves":"f3g5","mate":3}]}"#;
        let eval: LichessCloudEval = serde_json::from_str(json).unwrap();
        assert_eq!(eval.depth, 29);
        assert_eq!(eval.pvs[0].cp, Some(41));
        assert_eq!(eval.pvs[1].mate, Some(3));
        assert_eq!(eval.pvs[1].cp, None);
    }
}
