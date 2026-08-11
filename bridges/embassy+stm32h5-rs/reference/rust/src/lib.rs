//! Compiled reference examples for the embassy+stm32h5-rs bridge codegen prose.
//!
//! Each module mirrors the SHAPE of code `/spec-gen` emits for one peripheral
//! kind — a task fn carrying the explicit task-side type annotation plus the
//! entry-point construction — so `cargo build --target thumbv8m.main-none-eabihf`
//! type-checks the worked examples in `../codegen/rust/README.md`. A wrong
//! generic arity, `new` arg order, unbound interrupt, or raw-Peri-vs-handle
//! mistake fails this build. CI-only; never inlined.
//!
//! It is a `no_std` LIB (not a bin): a lib type-checks every example without
//! linking, so no `memory.x` / entry point / panic handler / runner is needed.
// A dependency declared but never imported is one the unit contributes to user
// projects with no compiled witness that it is needed or that its version is
// right. `deny` rather than `warn` so the build fails wherever it runs -- the
// reference-crate CI script uses a plain `cargo build`, and a warning nobody
// reads catches nothing. If a dep is genuinely used only under a cfg, add
// `use <crate> as _;` here rather than downgrading this.
#![deny(unused_crate_dependencies)]
#![no_std]

// One shared interrupt binding for the whole crate. (Per-module `bind_interrupts!`
// would emit duplicate interrupt-handler symbols within one crate.) Fully-qualified
// paths so the example modules can still be named `i2c` / `usart` without clashing
// with `embassy_stm32::i2c` etc.
embassy_stm32::bind_interrupts!(pub struct Irqs {
    I2C1_EV => embassy_stm32::i2c::EventInterruptHandler<embassy_stm32::peripherals::I2C1>;
    I2C1_ER => embassy_stm32::i2c::ErrorInterruptHandler<embassy_stm32::peripherals::I2C1>;
    USART3 => embassy_stm32::usart::InterruptHandler<embassy_stm32::peripherals::USART3>;
    GPDMA1_CHANNEL0 => embassy_stm32::dma::InterruptHandler<embassy_stm32::peripherals::GPDMA1_CH0>;
    GPDMA1_CHANNEL1 => embassy_stm32::dma::InterruptHandler<embassy_stm32::peripherals::GPDMA1_CH1>;
});

pub mod gpio;
pub mod i2c;
pub mod usart;
