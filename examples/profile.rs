//! Print the authenticated user's profile.
//!
//! Run with: `LICHESS_TOKEN=lip_xxx cargo run --example profile`

use litchee::LichessClient;

#[tokio::main]
async fn main() -> litchee::Result<()> {
    let token = std::env::var("LICHESS_TOKEN").expect("set the LICHESS_TOKEN environment variable");
    let client = LichessClient::builder().token(token).build()?;

    let me = client.account().profile().await?;
    println!("Logged in as {} <{}>", me.user.username, me.url);

    if let Some(perfs) = &me.user.perfs {
        if let Some(blitz) = &perfs.blitz {
            println!("Blitz rating: {}", blitz.rating);
        }
    }

    Ok(())
}
