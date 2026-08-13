//! One complete application, exercising every kind the codegen prose
//! documents. This is one file because `#[rtic::app]` is one-per-crate:
//! it generates an entry point and vector table, so a second app in the
//! same crate does not compile.
//!
//! Mirrors the shape `/spec-gen` emits from a module's `[rtic]` table:
//!
//!   [[rtic.tasks]]      -> the `#[task]` fns below
//!   [[rtic.resources]]  -> fields of `Shared` / `Local`
//!   [[rtic.channels]]   -> the `make_channel!` pair in `init`
//!
//! The `device` path and the `dispatchers` list are chip facts. Here they
//! come from the reference's compile vehicle; in a real project they come
//! from the chip binding.

#[rtic::app(device = lm3s6965, dispatchers = [UART0, UART1])]
mod app {
    use rtic_sync::{channel::{Receiver, Sender}, make_channel};

    /// `[[rtic.resources]]` with `sharing = "shared"`. Reachable from more
    /// than one task, so rtic applies the ceiling protocol to it.
    #[shared]
    struct Shared {
        counter: u32,
    }

    /// `[[rtic.resources]]` with `sharing = "local"`. Owned by exactly one
    /// task, so no lock exists or is needed.
    #[local]
    struct Local {
        toggle: bool,
        tx: Sender<'static, u32, 4>,
        rx: Receiver<'static, u32, 4>,
    }

    #[init]
    fn init(_cx: init::Context) -> (Shared, Local) {
        // `[[rtic.channels]]`: capacity is the second macro argument and is
        // fixed at compile time.
        let (tx, rx) = make_channel!(u32, 4);

        // A software task is started by its generated `spawn()`.
        drain::spawn().ok();

        (Shared { counter: 0 }, Local { toggle: false, tx, rx })
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
    #[task(binds = GPIOA, shared = [counter], local = [toggle], priority = 3)]
    fn sample(mut cx: sample::Context) {
        cx.shared.counter.lock(|counter: &mut u32| *counter += 1);

        // `#[local]` is the direct-access case, and the only one: no other
        // task can name this resource, so there is nothing to lock against.
        *cx.local.toggle = !*cx.local.toggle;
    }

    /// `kind = "software"`, `priority = 1`.
    ///
    /// An `async fn` — always, for software tasks. Same `.lock()` as
    /// above; emitting a direct access does not compile, and rtic names
    /// the offending type `counter_that_needs_to_be_locked` so the error
    /// says what to do.
    #[task(shared = [counter], priority = 1)]
    async fn drain(mut cx: drain::Context) {
        let value = cx.shared.counter.lock(|counter: &mut u32| *counter);
        let _ = value;
    }

    /// A second software task, showing the channel ends in use. Sending
    /// and receiving are both `.await` points, which is why channel work
    /// belongs in a software task and never in a hardware one.
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
