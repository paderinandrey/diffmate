use super::endpoint::FakeAnthropic;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

fn address(url: &str) -> &str {
    url.strip_prefix("http://").unwrap()
}

async fn send(url: &str, raw: &[u8]) -> Vec<u8> {
    let mut stream = TcpStream::connect(address(url)).await.unwrap();
    stream.write_all(raw).await.unwrap();
    let mut reply = Vec::new();
    stream.read_to_end(&mut reply).await.unwrap();
    reply
}

fn with_body(head: &str, body: &[u8]) -> Vec<u8> {
    let mut raw = head.as_bytes().to_vec();
    raw.extend_from_slice(body);
    raw
}

#[tokio::test]
async fn bodies_are_recorded_byte_for_byte_and_answered_with_400() {
    let endpoint = FakeAnthropic::start().await;
    let binary: Vec<u8> = (0..=255).cycle().take(300_000).collect();
    let head = format!(
        "POST /v1/messages?beta=true HTTP/1.1\r\nHost: x\r\nContent-Length: {}\r\n\r\n",
        binary.len()
    );
    let plain = send(&endpoint.url, &with_body(&head, &binary)).await;
    let chunk_head = "POST /chunked HTTP/1.1\r\nHost: x\r\nTransfer-Encoding: chunked\r\n\r\n";
    let chunked = "5\r\nhello\r\n6;ext=1\r\n world\r\n0\r\nTrailer: 1\r\n\r\n";
    let second = send(&endpoint.url, &with_body(chunk_head, chunked.as_bytes())).await;
    let third = send(
        &endpoint.url,
        b"HEAD /api/hello HTTP/1.1\r\nHost: x\r\n\r\n",
    )
    .await;

    for reply in [&plain, &second] {
        let text = String::from_utf8(reply.clone()).unwrap();
        assert!(text.starts_with("HTTP/1.1 400"), "{text}");
        assert!(text.ends_with("diffmate isolation harness\"}}"), "{text}");
    }
    let head_reply = String::from_utf8(third).unwrap();
    assert!(head_reply.starts_with("HTTP/1.1 400"), "{head_reply}");
    assert!(head_reply.ends_with("\r\n\r\n"), "{head_reply}");

    let seen = endpoint.requests();
    let summary: Vec<_> = seen.iter().map(|r| (&*r.method, &*r.path)).collect();
    let expected = [
        ("POST", "/v1/messages?beta=true"),
        ("POST", "/chunked"),
        ("HEAD", "/api/hello"),
    ];
    assert_eq!(summary, expected);
    assert_eq!(seen[0].body, binary);
    assert_eq!(seen[1].body, b"hello world");
    assert_eq!(seen[2].body, Vec::<u8>::new());
}

#[tokio::test]
async fn connections_are_counted_even_when_no_request_is_sent() {
    let endpoint = FakeAnthropic::start().await;
    for expected in 1..=2 {
        let mut stream = TcpStream::connect(address(&endpoint.url)).await.unwrap();
        stream.shutdown().await.unwrap();
        let mut rest = Vec::new();
        stream.read_to_end(&mut rest).await.unwrap();
        assert_eq!(rest, Vec::<u8>::new());
        assert_eq!(endpoint.connections(), expected);
    }
    assert!(endpoint.requests().is_empty());
}
