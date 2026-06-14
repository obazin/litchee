//! Resilience behaviour: the read timeout aborts a stalled connection but does
//! not cut off a healthy stream whose keep-alives arrive within the window.

use std::time::Duration;

use futures_util::StreamExt;
use litchee::LichessClient;
use litchee::error::LichessError;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn read_timeout_aborts_a_stalled_response() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/account"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"{"id":"me","username":"Me"}"#)
                .set_delay(Duration::from_secs(3)),
        )
        .mount(&server)
        .await;

    let client = LichessClient::builder()
        .base_url(&server.uri().parse().unwrap())
        .token("t")
        .read_timeout(Duration::from_millis(150))
        .build()
        .unwrap();

    let err = client.account().profile().await.unwrap_err();

    assert!(
        matches!(err, LichessError::Transport(_)),
        "expected a transport timeout, got {err:?}"
    );
}

#[tokio::test]
async fn keepalives_within_the_window_keep_a_stream_alive() {
    // A raw server: send a keep-alive blank line, pause for less than the read
    // timeout, then send a data line. The read timeout resets on each read, so
    // the gap must NOT abort the stream. (wiremock can't dribble bytes over time.)
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (mut sock, _) = listener.accept().await.unwrap();
        let mut scratch = [0u8; 1024];
        let _ = sock.read(&mut scratch).await; // consume the request
        sock.write_all(
            b"HTTP/1.1 200 OK\r\nContent-Type: application/x-ndjson\r\nTransfer-Encoding: chunked\r\n\r\n",
        )
        .await
        .unwrap();
        sock.write_all(b"1\r\n\n\r\n").await.unwrap(); // keep-alive (blank line)
        sock.flush().await.unwrap();
        tokio::time::sleep(Duration::from_millis(250)).await; // gap < read timeout
        sock.write_all(b"c\r\n{\"id\":\"g1\"}\n\r\n").await.unwrap();
        sock.write_all(b"0\r\n\r\n").await.unwrap();
        sock.flush().await.unwrap();
    });

    let client = LichessClient::builder()
        .base_url(&format!("http://{addr}").parse().unwrap())
        .token("t")
        .read_timeout(Duration::from_millis(600))
        .build()
        .unwrap();

    let games: Vec<_> = client.swiss().games("g1").await.unwrap().collect().await;

    assert_eq!(games.len(), 1);
    assert_eq!(games[0].as_ref().unwrap().id, "g1");
}
