//! GPIO reference — mirrors the bridge README's GPIO output example. A GPIO
//! pin (mode = "output") IS pre-constructed as a handle (unlike alternate-
//! function bus pins, which pass as raw `Peri`).
use embassy_stm32::gpio::{Level, Output, Speed};

#[embassy_executor::task]
async fn blink(mut led: Output<'static>) {
    led.toggle();
}

pub fn init(spawner: &embassy_executor::Spawner, led: Output<'static>) {
    spawner.spawn(blink(led).unwrap());
}

pub fn build(spawner: &embassy_executor::Spawner) {
    let p = embassy_stm32::init(embassy_stm32::Config::default());
    let led = Output::new(p.PB0, Level::Low, Speed::Low);
    init(spawner, led);
}
