//! `endpoint` — binding an Endpoint for each `identity` / `relay` /
//! `address_lookup` combination the `[iroh]` table allows. Mirrors the
//! README's endpoint-emission section. Types annotated explicitly so the
//! compiler checks them.
use iroh::endpoint::{presets, BindError};
use iroh::{Endpoint, EndpointAddr, EndpointId, SecretKey};
use iroh_mdns_address_lookup::MdnsAddressLookup;

/// identity = "ephemeral", relay = "n0", address_lookup = ["dns"]
pub async fn bind_ephemeral() -> Result<Endpoint, BindError> {
    let endpoint: Endpoint = Endpoint::bind(presets::N0).await?;
    endpoint.online().await;
    Ok(endpoint)
}

/// relay = "disabled" — direct paths only, no relay assistance.
pub async fn bind_no_relay() -> Result<Endpoint, BindError> {
    let endpoint: Endpoint = Endpoint::bind(presets::N0DisableRelay).await?;
    Ok(endpoint)
}

/// identity = "persisted", address_lookup = ["dns", "mdns"]
pub async fn bind_persisted(secret: SecretKey) -> Result<Endpoint, BindError> {
    let endpoint: Endpoint = Endpoint::builder(presets::N0)
        .secret_key(secret)
        .address_lookup(MdnsAddressLookup::builder())
        .bind()
        .await?;
    endpoint.online().await;
    Ok(endpoint)
}

/// identity = "persisted" — the load-or-create half. Callers supply the
/// stored bytes; where they are stored is the project's decision, not
/// this block's.
pub fn load_or_generate(stored: Option<[u8; 32]>) -> (SecretKey, [u8; 32]) {
    let secret: SecretKey = match stored {
        Some(bytes) => SecretKey::from_bytes(&bytes),
        None => SecretKey::generate(),
    };
    let to_store: [u8; 32] = secret.to_bytes();
    (secret, to_store)
}

/// The two identifiers a caller needs out of a bound endpoint.
pub fn identifiers(endpoint: &Endpoint) -> (EndpointId, EndpointAddr) {
    let id: EndpointId = endpoint.id();
    let addr: EndpointAddr = endpoint.addr();
    (id, addr)
}

pub async fn shutdown(endpoint: &Endpoint) {
    endpoint.close().await;
}
