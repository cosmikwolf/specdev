//! `stream_uni` — `kind = "uni"`: a one-way push stream for continuous
//! flow, with `framing = "length_prefixed"`. No return path is opened,
//! so nothing allocates a reply channel that is never read.
//! Mirrors the README's uni worked example.
use iroh::endpoint::Connection;
use serde::{Deserialize, Serialize};

/// Stands in for the domain type named by `carries`.
#[derive(Debug, Serialize, Deserialize)]
pub struct TelemetryFrame {
    pub seq: u64,
    pub samples: Vec<f32>,
}

pub const MAX_FRAME: usize = 64 * 1024;

/// direction = "outbound": open once, then push frames until told to stop.
pub async fn push(
    conn: &Connection,
    frames: &[TelemetryFrame],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut send = conn.open_uni().await?;

    for frame in frames {
        let encoded: Vec<u8> = postcard::to_stdvec(frame)?;
        let len: u32 = encoded.len() as u32;
        send.write_all(&len.to_le_bytes()).await?;
        send.write_all(&encoded).await?;
    }

    send.finish()?;
    Ok(())
}

/// direction = "inbound": accept once, then decode frames until the peer
/// finishes the stream. A short read on the length prefix is the normal
/// end of stream, not an error.
pub async fn drain(conn: &Connection) -> Result<Vec<TelemetryFrame>, Box<dyn std::error::Error>> {
    let mut recv = conn.accept_uni().await?;
    let mut out: Vec<TelemetryFrame> = Vec::new();

    loop {
        let mut len_bytes = [0u8; 4];
        if recv.read_exact(&mut len_bytes).await.is_err() {
            break;
        }
        let len: usize = u32::from_le_bytes(len_bytes) as usize;
        if len > MAX_FRAME {
            return Err("frame exceeds MAX_FRAME".into());
        }
        let mut body = vec![0u8; len];
        recv.read_exact(&mut body).await?;
        out.push(postcard::from_bytes(&body)?);
    }

    Ok(out)
}
