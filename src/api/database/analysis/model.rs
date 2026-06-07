//! DTOs for the Analysis concern.

use serde::{Deserialize, Serialize};

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
