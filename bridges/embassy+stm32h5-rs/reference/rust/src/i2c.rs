//! I²C reference — mirrors the bridge README "Async I²C — worked entry-point
//! example". The explicit `I2c<'static, Async, Master>` annotation on the task
//! fn is what makes the compiler check the 3-generic arity (a bare
//! `let i2c = I2c::new(...)` with inference would NOT catch a wrong table type).
use embassy_stm32::i2c;
use embassy_stm32::i2c::{I2c, Master};
use embassy_stm32::mode::Async;

use crate::Irqs;

#[embassy_executor::task]
async fn poll(mut i2c: I2c<'static, Async, Master>) {
    let mut buf = [0u8; 1];
    let _ = i2c.write_read(0x77, &[0x0D], &mut buf).await;
}

pub fn init(spawner: &embassy_executor::Spawner, i2c: I2c<'static, Async, Master>) {
    spawner.spawn(poll(i2c).unwrap());
}

/// Mirrors the entry-point construction (a plain fn since a lib has no
/// `#[embassy_executor::main]`).
pub fn build(spawner: &embassy_executor::Spawner) {
    let p = embassy_stm32::init(embassy_stm32::Config::default());

    let mut cfg = i2c::Config::default();
    cfg.sda_pullup = true;
    cfg.scl_pullup = true;

    let i2c = i2c::I2c::new(
        p.I2C1,
        p.PB8,        // SCL — raw Peri, not Output::new
        p.PB9,        // SDA — raw Peri
        p.GPDMA1_CH0, // tx dma
        p.GPDMA1_CH1, // rx dma
        Irqs,
        cfg,
    );
    init(spawner, i2c);
}
