//! One complete application, exercising every kind the codegen prose
//! documents. This is one file because `#[rtic::app]` is one-per-crate:
//! it generates an entry point and vector table, so a second app in the
//! same crate does not compile.
//!
//! Mirrors the shape `/spec-gen` emits from a module's `[rtic]` table:
//!
//!   [[rtic.tasks]]      -> the `#[task]` fns below
//!   [[rtic.resources]]  -> fields of `Shared` / `Local`, or an inline local
//!   [[rtic.channels]]   -> form A (`make_channel!` in `init`) AND form B
//!                          (a `static Channel`), because the prose documents
//!                          both and a form nobody compiles is unchecked.
//!                          A real application uses exactly one.
//!
//! The `device` path and the `dispatchers` list are chip facts. Here they
//! come from the reference's compile vehicle; in a real project they come
//! from the chip binding.

#[rtic::app(device = lm3s6965, dispatchers = [UART0, UART1])]
mod app {
    use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
    use embassy_sync::channel::Channel;
    use rtic_sync::{channel::{Receiver, Sender}, make_channel};

    /// Form B channel: `[[rtic.channels]] name = "events", payload_type =
    /// "u32", capacity = 4`, emitted when the manifest holds `embassy-sync`.
    /// A static at module scope; every task can reach it, and it has a sync
    /// send, which is what lets the hardware task below feed the software
    /// task without either end being a resource.
    static EVENTS: Channel<CriticalSectionRawMutex, u32, 4> = Channel::new();

    /// `[[rtic.resources]]` with `sharing = "shared"`. Reachable from more
    /// than one task, so rtic applies the ceiling protocol to it.
    ///
    /// `table` is the inline-local pattern from prose §2: a large value
    /// declared on `init` lives in a static from the start, and its
    /// `&'static mut` is then stored here so two priorities can lock it.
    #[shared]
    struct Shared {
        counter: u32,
        table: &'static mut [u16; 64],
    }

    /// `[[rtic.resources]]` with `sharing = "local"`. Owned by exactly one
    /// task, so no lock exists or is needed.
    #[local]
    struct Local {
        toggle: bool,
        tx: Sender<'static, u32, 4>,
        rx: Receiver<'static, u32, 4>,
    }

    /// The inline local on `init`: `table` never touches the init stack
    /// frame. Its initializer must be a constant expression.
    #[init(local = [table: [u16; 64] = [0; 64]])]
    fn init(cx: init::Context) -> (Shared, Local) {
        let table: &'static mut [u16; 64] = cx.local.table;

        // Form A channel (`rtic-sync`): capacity is the second macro
        // argument and is fixed at compile time.
        let (tx, rx) = make_channel!(u32, 4);

        // A software task is started by its generated `spawn()`.
        drain::spawn().ok();

        (Shared { counter: 0, table }, Local { toggle: false, tx, rx })
    }

    /// `kind = "hardware"`, `binds = "GPIOA"`, `priority = 3`.
    ///
    /// A plain `fn` — never `async`.
    ///
    /// `counter` is `#[shared]`, so it arrives as a lock proxy and the
    /// access goes through `.lock()`. That holds even here, at the highest
    /// priority of any task touching it: rtic 2.x applies no
    /// direct-access optimization to `#[shared]`, not for the ceiling
    /// task, not for a sole accessor, not for same-priority peers.
    /// Verified by compiling each of those three cases.
    ///
    /// `ticks` is a task-declared inline local (prose §2): private to this
    /// task, static placement, no `Local` field needed.
    ///
    /// The form-B `try_send` is the sync path a hardware task may use; a
    /// full channel is a runtime state, counted here rather than unwrapped.
    #[task(binds = GPIOA, shared = [counter, table], local = [toggle, ticks: u32 = 0], priority = 3)]
    fn sample(mut cx: sample::Context) {
        cx.shared.counter.lock(|counter: &mut u32| *counter += 1);
        cx.shared.table.lock(|table: &mut &'static mut [u16; 64]| table[0] = table[0].wrapping_add(1));

        // `#[local]` is the direct-access case, and the only one: no other
        // task can name this resource, so there is nothing to lock against.
        *cx.local.toggle = !*cx.local.toggle;
        *cx.local.ticks += 1;

        if EVENTS.try_send(*cx.local.ticks).is_err() {
            // full: record the overflow and move on
        }
    }

    /// `kind = "software"`, `priority = 1`.
    ///
    /// An `async fn` — always, for software tasks. Same `.lock()` as
    /// above; emitting a direct access does not compile, and rtic names
    /// the offending type `counter_that_needs_to_be_locked` so the error
    /// says what to do.
    ///
    /// Also the form-B consumer: `receive()` is a future, so it belongs
    /// here and never in a hardware task.
    #[task(shared = [counter, table], priority = 1)]
    async fn drain(mut cx: drain::Context) {
        let value = cx.shared.counter.lock(|counter: &mut u32| *counter);
        let _ = value;
        let first = cx.shared.table.lock(|table: &mut &'static mut [u16; 64]| table[0]);
        let _ = first;

        let event = EVENTS.receive().await;
        let _ = event;
    }

    /// A second software task, showing the form-A channel ends in use.
    /// Sending and receiving are both `.await` points, which is why form-A
    /// channel work belongs in a software task and never in a hardware one.
    #[task(local = [tx, rx], priority = 2)]
    async fn pump(cx: pump::Context) {
        let tx: &mut Sender<'static, u32, 4> = cx.local.tx;
        let rx: &mut Receiver<'static, u32, 4> = cx.local.rx;

        tx.send(1).await.ok();
        if let Ok(value) = rx.recv().await {
            let _ = value;
        }
    }

    /// `kind = "idle"`. Runs when nothing else is runnable, never returns,
    /// and takes no priority — it is always 0.
    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        loop {
            cortex_m::asm::wfi();
        }
    }
}
