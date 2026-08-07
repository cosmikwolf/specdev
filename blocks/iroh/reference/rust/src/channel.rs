//! `channel` — the thread-crossing surface: one bounded tokio channel per
//! `[[iroh.channels]]` entry, plus the three `overflow` policies.
//! Mirrors the README's channel-emission section.
//!
//! The handle is what a caller holds. It exposes no Endpoint, no
//! Connection and no async machinery beyond the channel ends themselves,
//! so the runtime can be driven from a thread that knows nothing about
//! iroh.
use std::collections::VecDeque;
use tokio::sync::mpsc;

/// Stands in for the `payload_type` of a `direction = "to_runtime"` channel.
#[derive(Debug)]
pub enum Command {
    Connect(String),
    Disconnect,
    Shutdown,
}

/// Stands in for the `payload_type` of a `direction = "from_runtime"` channel.
#[derive(Debug)]
pub enum Event {
    Connected,
    Frame(Vec<u8>),
    Disconnected,
}

/// What the caller keeps: the sending half of every `to_runtime` channel
/// and the receiving half of every `from_runtime` channel.
pub struct NetHandle {
    pub commands: mpsc::Sender<Command>,
    pub events: mpsc::Receiver<Event>,
}

/// What the runtime task keeps: the mirror image.
pub struct NetTaskEnds {
    pub commands: mpsc::Receiver<Command>,
    pub events: mpsc::Sender<Event>,
}

pub fn build(command_capacity: usize, event_capacity: usize) -> (NetHandle, NetTaskEnds) {
    let (command_tx, command_rx) = mpsc::channel::<Command>(command_capacity);
    let (event_tx, event_rx) = mpsc::channel::<Event>(event_capacity);
    (
        NetHandle {
            commands: command_tx,
            events: event_rx,
        },
        NetTaskEnds {
            commands: command_rx,
            events: event_tx,
        },
    )
}

/// overflow = "backpressure" — await a free slot. Losing a command
/// silently changes program behaviour, so commands must never be dropped.
pub async fn send_backpressure(
    tx: &mpsc::Sender<Command>,
    cmd: Command,
) -> Result<(), mpsc::error::SendError<Command>> {
    tx.send(cmd).await
}

/// overflow = "drop_newest" — reject the incoming message, keep the queue.
pub fn send_drop_newest(tx: &mpsc::Sender<Event>, ev: Event) -> bool {
    tx.try_send(ev).is_ok()
}

/// overflow = "drop_oldest" — a bounded producer-side ring in front of the
/// channel. A sender cannot evict from a tokio channel, so the eviction has
/// to happen before the message is handed over: push into the ring, evicting
/// the oldest when full, then flush whatever the channel will currently take.
pub struct DropOldest {
    ring: VecDeque<Event>,
    capacity: usize,
    pub dropped: u64,
}

impl DropOldest {
    pub fn new(capacity: usize) -> Self {
        Self {
            ring: VecDeque::with_capacity(capacity),
            capacity,
            dropped: 0,
        }
    }

    pub fn push(&mut self, tx: &mpsc::Sender<Event>, ev: Event) {
        if self.ring.len() == self.capacity {
            self.ring.pop_front();
            self.dropped += 1;
        }
        self.ring.push_back(ev);
        self.flush(tx);
    }

    fn flush(&mut self, tx: &mpsc::Sender<Event>) {
        while let Some(ev) = self.ring.pop_front() {
            if let Err(err) = tx.try_send(ev) {
                // Channel full or closed: put it back and stop for now.
                if let mpsc::error::TrySendError::Full(ev) = err {
                    self.ring.push_front(ev);
                }
                return;
            }
        }
    }
}
