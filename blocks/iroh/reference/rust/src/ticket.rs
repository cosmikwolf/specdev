//! `ticket` — sharing an EndpointAddr as text.
//!
//! iroh 1.0.3 ships NO ticket type: `iroh_base` exports `EndpointAddr`,
//! `key` and `relay_url` only. `EndpointAddr` does derive Serialize /
//! Deserialize, so a ticket is "encode it and choose a text encoding".
//! This reference hand-rolls hex to stay dependency-free; a project that
//! wants shorter, case-insensitive tickets should reach for base32 via
//! `data-encoding` (see recommended_external_deps).
use iroh::EndpointAddr;

pub fn encode(addr: &EndpointAddr) -> Result<String, postcard::Error> {
    let bytes: Vec<u8> = postcard::to_stdvec(addr)?;
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(nibble(byte >> 4));
        out.push(nibble(byte & 0x0f));
    }
    Ok(out)
}

pub fn decode(ticket: &str) -> Result<EndpointAddr, Box<dyn std::error::Error>> {
    let trimmed = ticket.trim();
    if trimmed.len() % 2 != 0 {
        return Err("ticket has an odd number of hex digits".into());
    }
    let mut bytes: Vec<u8> = Vec::with_capacity(trimmed.len() / 2);
    let chars: Vec<char> = trimmed.chars().collect();
    for pair in chars.chunks(2) {
        let hi = un_nibble(pair[0])?;
        let lo = un_nibble(pair[1])?;
        bytes.push((hi << 4) | lo);
    }
    let addr: EndpointAddr = postcard::from_bytes(&bytes)?;
    Ok(addr)
}

fn nibble(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        _ => (b'a' + value - 10) as char,
    }
}

fn un_nibble(c: char) -> Result<u8, Box<dyn std::error::Error>> {
    c.to_digit(16)
        .map(|d| d as u8)
        .ok_or_else(|| "ticket contains a non-hex character".into())
}
