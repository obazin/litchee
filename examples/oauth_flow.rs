//! End-to-end "Log in with Lichess" demo.
//!
//! This example walks the full PKCE `OAuth2` Authorization Code flow, then uses
//! the freshly minted token to show the signed-in user's data:
//!
//! 1. Build an authorization URL (with a PKCE verifier + CSRF `state`).
//! 2. Open it in the browser so the user can approve on lichess.org.
//! 3. Catch the redirect on a tiny local HTTP listener and read the `code`.
//! 4. Exchange the `code` for an access token.
//! 5. List the 10 most recent games, the 10 most recent puzzle attempts, and
//!    the studies the user owns or contributes to.
//!
//! Register a public OAuth app is *not* required — Lichess accepts any
//! `client_id` for public PKCE clients. Just make sure the redirect URI below
//! matches what you pass.
//!
//! Run with:
//! ```text
//! cargo run --example oauth_flow
//! # or override the app identity:
//! LICHESS_CLIENT_ID=com.example.litchee cargo run --example oauth_flow
//! ```

use std::error::Error;

use futures_util::StreamExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{Duration, timeout};
use url::Url;

use litchee::LichessClient;
use litchee::api::auth::oauth::{AuthorizationRequest, CodeExchange, Scope};

/// Where the browser is sent back after the user approves access.
const REDIRECT_URI: &str = "http://localhost:8080/callback";
/// The local address the listener binds to (must match `REDIRECT_URI`).
const BIND_ADDR: &str = "127.0.0.1:8080";
/// Fallback application identifier for the OAuth client.
const DEFAULT_CLIENT_ID: &str = "tech.seventhrank.litchee.example";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client_id =
        std::env::var("LICHESS_CLIENT_ID").unwrap_or_else(|_| DEFAULT_CLIENT_ID.to_owned());

    // An unauthenticated client is enough to build the authorization URL.
    let client = LichessClient::builder().build()?;
    let token = run_login(&client, &client_id).await?;

    // Re-build the client, this time carrying the bearer token.
    let client = LichessClient::builder()
        .token(token.access_token.into_inner())
        .build()?;

    let me = client.account().profile().await?;
    println!("\n✅ Signed in as {} ({})\n", me.user.username, me.url);

    show_recent_games(&client, &me.user.username).await?;
    show_recent_puzzles(&client).await?;
    show_studies(&client, &me.user.username).await?;

    Ok(())
}

/// Drives the interactive half of the PKCE flow and returns the access token.
async fn run_login(
    client: &LichessClient,
    client_id: &str,
) -> Result<litchee::api::auth::oauth::LichessToken, Box<dyn Error>> {
    // Ask for read access to puzzle activity and (private) studies. Exporting a
    // user's own games needs no special scope.
    let scopes = [Scope::PuzzleRead, Scope::StudyRead];
    let auth = client.oauth().authorization_url(&AuthorizationRequest {
        client_id,
        redirect_uri: REDIRECT_URI,
        scopes: &scopes,
        username_hint: None,
    })?;

    println!("Opening your browser to authorize litchee…");
    println!(
        "If it doesn't open, paste this URL manually:\n\n  {}\n",
        auth.url
    );
    open_in_browser(auth.url.as_str());

    let code = wait_for_redirect(&auth.state).await?;
    println!("Authorization received — exchanging the code for a token…");

    let token = client
        .oauth()
        .exchange_code(&CodeExchange {
            code: &code,
            code_verifier: &auth.verifier,
            redirect_uri: REDIRECT_URI,
            client_id,
        })
        .await?;
    Ok(token)
}

/// Best-effort attempt to open `url` in the platform's default browser.
fn open_in_browser(url: &str) {
    #[cfg(target_os = "macos")]
    let program = "open";
    #[cfg(target_os = "windows")]
    let program = "explorer";
    #[cfg(all(unix, not(target_os = "macos")))]
    let program = "xdg-open";

    if let Err(err) = std::process::Command::new(program).arg(url).spawn() {
        eprintln!("(couldn't launch a browser automatically: {err})");
    }
}

/// How one inbound connection was classified.
enum Callback {
    /// The real redirect, carrying the authorization `code`.
    Code(String),
    /// The provider reported an explicit authorization error.
    Denied(String),
    /// Not the callback we want (favicon, browser pre-connect, wrong/missing
    /// `state`); answer politely and keep waiting.
    Ignore,
}

/// Waits for the OAuth redirect, validating the CSRF `state`, and returns the
/// authorization `code`.
///
/// Browsers routinely open speculative/pre-connect sockets and fetch
/// `/favicon.ico`, so a single `accept()` is unreliable: the first connection
/// is often not the callback. This loops, ignoring noise until the genuine
/// redirect arrives.
async fn wait_for_redirect(expected_state: &str) -> Result<String, Box<dyn Error>> {
    let listener = TcpListener::bind(BIND_ADDR).await?;
    println!("Waiting for the Lichess redirect on http://{BIND_ADDR} …");

    loop {
        let (mut stream, _) = listener.accept().await?;
        let target = read_request_target(&mut stream).await;
        match classify_request(&target, expected_state) {
            Callback::Code(code) => {
                let body = "litchee: authorization complete — you can close this tab.";
                write_http_response(&mut stream, body).await?;
                return Ok(code);
            }
            Callback::Denied(error) => {
                let body = "litchee: authorization failed — check the terminal.";
                write_http_response(&mut stream, body).await?;
                return Err(error.into());
            }
            Callback::Ignore => {
                let _ =
                    write_http_response(&mut stream, "litchee: waiting for authorization…").await;
            }
        }
    }
}

/// Reads the request line and returns its target (e.g. `/callback?code=…`).
///
/// Returns an empty string on a read error, timeout, or a connection that
/// sends no data (a pre-connect), so the caller simply ignores it instead of
/// hanging on `read`.
async fn read_request_target(stream: &mut TcpStream) -> String {
    let mut buf = [0u8; 2048];
    let Ok(Ok(n)) = timeout(Duration::from_secs(5), stream.read(&mut buf)).await else {
        return String::new();
    };
    let request = String::from_utf8_lossy(&buf[..n]);
    let first_line = request.lines().next().unwrap_or_default();
    // Request line shape: "GET /callback?... HTTP/1.1".
    first_line
        .split_whitespace()
        .nth(1)
        .unwrap_or_default()
        .to_owned()
}

/// Classifies a request target as the callback, an explicit denial, or noise.
fn classify_request(target: &str, expected_state: &str) -> Callback {
    let Ok(url) = Url::parse(&format!("http://localhost{target}")) else {
        return Callback::Ignore;
    };
    let mut code = None;
    let mut state = None;
    for (key, value) in url.query_pairs() {
        match key.as_ref() {
            "code" => code = Some(value.into_owned()),
            "state" => state = Some(value.into_owned()),
            "error" => return Callback::Denied(format!("authorization denied: {value}")),
            _ => {}
        }
    }
    // Ignore anything whose state doesn't match (including no-query favicon /
    // pre-connect requests) rather than aborting an otherwise-valid login.
    match code {
        Some(code) if state.as_deref() == Some(expected_state) => Callback::Code(code),
        _ => Callback::Ignore,
    }
}

/// Writes a minimal HTTP 200 response so the browser shows a friendly message.
async fn write_http_response(stream: &mut TcpStream, body: &str) -> Result<(), Box<dyn Error>> {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;
    Ok(())
}

/// Prints the 10 most recently played games for `username`.
async fn show_recent_games(client: &LichessClient, username: &str) -> Result<(), Box<dyn Error>> {
    println!("── 10 most recent games ──");
    let mut games = client
        .games()
        .export_user(username)
        .max(10)
        .stream()
        .await?;
    let mut count = 0;
    while let Some(game) = games.next().await {
        let game = game?;
        let speed = game
            .speed
            .map_or_else(|| "?".to_owned(), |s| format!("{s:?}"));
        let winner = game
            .winner
            .map_or_else(|| "draw/ongoing".to_owned(), |c| format!("{c:?}"));
        println!("  {} [{speed}] winner: {winner}", game.id);
        count += 1;
    }
    if count == 0 {
        println!("  (no games found)");
    }
    println!();
    Ok(())
}

/// Prints the 10 most recent puzzle attempts (requires `puzzle:read`).
async fn show_recent_puzzles(client: &LichessClient) -> Result<(), Box<dyn Error>> {
    println!("── 10 most recent puzzle attempts ──");
    let mut activity = client.puzzles().activity(Some(10)).await?;
    let mut count = 0;
    while let Some(entry) = activity.next().await {
        let entry = entry?;
        let result = if entry.win {
            "✔ solved"
        } else {
            "✘ failed"
        };
        let rating = entry
            .puzzle
            .rating
            .map_or_else(|| "?".to_owned(), |r| r.to_string());
        println!("  {} (rating {rating}) — {result}", entry.puzzle.id);
        count += 1;
    }
    if count == 0 {
        println!("  (no puzzle activity found)");
    }
    println!();
    Ok(())
}

/// Prints the studies the user owns or contributes to (requires `study:read`
/// to include private studies).
async fn show_studies(client: &LichessClient, username: &str) -> Result<(), Box<dyn Error>> {
    println!("── studies you own or take part in ──");
    let mut studies = client.studies().list_metadata(username).await?;
    let mut count = 0;
    while let Some(study) = studies.next().await {
        let study = study?;
        println!("  {} — {}", study.id, study.name);
        count += 1;
    }
    if count == 0 {
        println!("  (no studies found)");
    }
    println!();
    Ok(())
}
