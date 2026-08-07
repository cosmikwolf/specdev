//! `stream_bidi` — `kind = "bidi"`: a paired send/recv stream for
//! request/response traffic, with `framing = "length_prefixed"`.
//! Mirrors the README's bidi worked example.
use iroh::endpoint::Connection;
use serde::{Deserialize, Serialize};

/// Stands in for the domain type named by `carries`. The real type is
/// owned by the project's domain spec, never by this block.
#[derive(Debug, Serialize, Deserialize)]
pub enum Signal {
    Ping,
    Colour(u8, u8, u8),
}

/// Cap on a single decoded message. Refuse rather than allocate what a
/// peer claims — the length prefix is attacker-controlled.
pub const MAX_FRAME: usize = 64 * 1024;

/// direction = "outbound": open the stream, write one message, read the reply.
pub async fn request(conn: &Connection, msg: &Signal) -> Result<Signal, Box<dyn std::error::Error>> {
    let (mut send, mut recv) = conn.open_bi().await?;

    let encoded: Vec<u8> = postcard::to_stdvec(msg)?;
    let len: u32 = encoded.len() as u32;
    send.write_all(&len.to_le_bytes()).await?;
    send.write_all(&encoded).await?;
    send.finish()?;

    let reply: Vec<u8> = recv.read_to_end(MAX_FRAME).await?;
    let decoded: Signal = postcard::from_bytes(&reply)?;
    Ok(decoded)
}

/// direction = "inbound": accept the stream, read one message, answer it.
pub async fn respond(conn: &Connection) -> Result<Signal, Box<dyn std::error::Error>> {
    let (mut send, mut recv) = conn.accept_bi().await?;

    let mut len_bytes = [0u8; 4];
    recv.read_exact(&mut len_bytes).await?;
    let len: usize = u32::from_le_bytes(len_bytes) as usize;
    if len > MAX_FRAME {
        return Err("frame exceeds MAX_FRAME".into());
    }
    let mut body = vec![0u8; len];
    recv.read_exact(&mut body).await?;
    let decoded: Signal = postcard::from_bytes(&body)?;

    let encoded: Vec<u8> = postcard::to_stdvec(&decoded)?;
    send.write_all(&encoded).await?;
    send.finish()?;
    Ok(decoded)
}
