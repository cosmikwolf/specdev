//! `protocol` — the ALPN handler and Router for `role = "accept"` /
//! `role = "both"`, plus the dial path for `role = "connect"`.
//! Mirrors the README's protocol-emission section.
use iroh::endpoint::{Connection, ConnectError};
use iroh::protocol::{AcceptError, ProtocolHandler, Router};
use iroh::{Endpoint, EndpointAddr};

/// One ALPN string per endpoint. Byte-identical on both sides or the
/// handshake fails.
pub const ALPN: &[u8] = b"specdev/iroh-reference/0";

#[derive(Debug, Clone)]
pub struct Handler;

impl ProtocolHandler for Handler {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        // Per-connection work goes here; the reference only proves the shape.
        let _peer = connection.remote_id();
        connection.closed().await;
        Ok(())
    }
}

/// role = "accept" | "both" — spawn the accept loop.
pub fn spawn_router(endpoint: Endpoint) -> Router {
    Router::builder(endpoint).accept(ALPN, Handler).spawn()
}

/// role = "connect" | "both" — dial a peer by address.
pub async fn dial(endpoint: &Endpoint, addr: EndpointAddr) -> Result<Connection, ConnectError> {
    let connection: Connection = endpoint.connect(addr, ALPN).await?;
    Ok(connection)
}

/// Routers abort on drop; shut down explicitly so in-flight connections
/// close cleanly.
pub async fn shutdown(router: &Router) {
    let _ = router.shutdown().await;
}
