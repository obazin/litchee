//! Stream the current Lichess TV featured game (no authentication needed).
//!
//! Run with: `cargo run --example tv_feed`

// CLAUDE.md rule 2 scopes the 20-line function cap to src/; examples sit
// outside that boundary, so a walkthrough may read linearly top-to-bottom.
#![allow(clippy::too_many_lines)]

use futures_util::StreamExt;
use litchee::LichessClient;

#[tokio::main]
async fn main() -> litchee::Result<()> {
    let client = LichessClient::new();
    let mut feed = client.tv().feed().await?;

    // Print the first few events, then stop.
    let mut remaining = 5;
    while let Some(event) = feed.next().await {
        println!("{:?}", event?);
        remaining -= 1;
        if remaining == 0 {
            break;
        }
    }

    Ok(())
}
