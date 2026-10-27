use super::endpoint::Recorded;
use std::io;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

const REPLY: &str = r#"{"type":"error","error":{"type":"invalid_request_error","message":"diffmate isolation harness"}}"#;

type Reader = BufReader<TcpStream>;

async fn line(reader: &mut Reader) -> io::Result<String> {
    let mut text = String::new();
    if reader.read_line(&mut text).await? == 0 {
        return Err(io::ErrorKind::UnexpectedEof.into());
    }
    Ok(text.trim_end().to_string())
}

async fn chunked(reader: &mut Reader) -> io::Result<Vec<u8>> {
    let mut body = Vec::new();
    loop {
        let size_line = line(reader).await?;
        let digits = size_line.split(';').next().unwrap_or_default().trim();
        let size = usize::from_str_radix(digits, 16).map_err(io::Error::other)?;
        if size == 0 {
            break;
        }
        let start = body.len();
        body.resize(start + size, 0);
        reader.read_exact(&mut body[start..]).await?;
        line(reader).await?;
    }
    while !line(reader).await?.is_empty() {}
    Ok(body)
}

pub(super) async fn read_request(reader: &mut Reader) -> io::Result<Recorded> {
    let request_line = line(reader).await?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default().to_string();
    let path = parts.next().unwrap_or_default().to_string();
    let (mut length, mut is_chunked) = (0, false);
    loop {
        let header = line(reader).await?;
        if header.is_empty() {
            break;
        }
        let Some((name, value)) = header.split_once(':') else {
            continue;
        };
        let value = value.trim().to_ascii_lowercase();
        match name.trim().to_ascii_lowercase().as_str() {
            "content-length" => length = value.parse().map_err(io::Error::other)?,
            "transfer-encoding" => is_chunked = value.contains("chunked"),
            _ => {}
        }
    }
    let body = if is_chunked {
        chunked(reader).await?
    } else {
        let mut body = vec![0; length];
        reader.read_exact(&mut body).await?;
        body
    };
    Ok(Recorded { method, path, body })
}

pub(super) async fn reply(reader: &mut Reader, method: &str) -> io::Result<()> {
    let head = format!(
        "HTTP/1.1 400 Bad Request\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
        REPLY.len()
    );
    let stream = reader.get_mut();
    stream.write_all(head.as_bytes()).await?;
    if method != "HEAD" {
        stream.write_all(REPLY.as_bytes()).await?;
    }
    stream.shutdown().await
}
