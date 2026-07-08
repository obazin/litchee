//! Shared, cross-cutting data types (`Lichess*` DTOs) used by many concerns.
//!
//! Types that are specific to a single concern live with that concern; the ones
//! here — players, titles, perfs, and a handful of primitives — are reused
//! widely enough to warrant a common home.

mod common;
mod game_export;
mod perf;
mod title;
mod user;

pub use common::{LichessColor, LichessOk, LichessSpeed, LichessVariantKey};
pub use game_export::GameExportOptions;
pub use perf::{LichessPerf, LichessPerfs, LichessPuzzleModePerf};
pub use title::LichessTitle;
pub use user::{
    LichessCount, LichessLightUser, LichessPlayTime, LichessProfile, LichessStreamerChannel,
    LichessStreamerInfo, LichessUser, LichessUserExtended,
};
