//! `channel` — the three `[[embassy.channels]]` kinds, each a `static` from
//! embassy-sync with the documented generic arity (Channel=3, Watch=3,
//! Signal=2). Mirrors the README's channel-emission table + worked example.
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::signal::Signal;
use embassy_sync::watch::Watch;

// kind = "bounded"
static EVENTS: Channel<CriticalSectionRawMutex, u8, 16> = Channel::new();
// kind = "watch"
static STATE: Watch<CriticalSectionRawMutex, u8, 4> = Watch::new();
// kind = "signal"
static FLAG: Signal<CriticalSectionRawMutex, u8> = Signal::new();

pub fn touch() {
    let _ = EVENTS.try_send(0);
    let _ = STATE.sender();
    FLAG.signal(1);
}
