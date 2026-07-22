//! The exposed API surface the FRB generator scans. Pure-function processing
//! only — no platform-bound I/O, no `std::thread::spawn` — so the crate
//! compiles for native targets and `wasm32-unknown-unknown` alike.
use crate::frb_generated::StreamSink;
use flutter_rust_bridge::frb;

#[frb(init)]
pub fn init_app() {
    flutter_rust_bridge::setup_default_user_utils();
}

/// Synchronous call: returns on the calling thread, no executor hop.
#[frb(sync)]
pub fn greet(name: String) -> String {
    format!("Hello, {name}!")
}

/// Asynchronous call: the default FRB shape; runs off the UI thread.
pub async fn checksum(data: Vec<u8>) -> u32 {
    data.iter().map(|b| u32::from(*b)).sum()
}

/// Stream: Rust pushes values, Dart consumes a `Stream<int>`.
pub fn count_up_to(sink: StreamSink<u32>, limit: u32) {
    for value in 0..limit {
        let _ = sink.add(value);
    }
}
