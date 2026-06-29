//! USART reference — mirrors the bridge README's async-USART worked example.
//! `Uart` is 2-generic (`Uart<'static, Async>`) and `Uart::new` returns
//! `Result` (note the `.unwrap()`), unlike I²C's `Self`.
use embassy_stm32::mode::Async;
use embassy_stm32::usart::Uart;

use crate::Irqs;

#[embassy_executor::task]
async fn read_uart(mut uart: Uart<'static, Async>) {
    let mut b = [0u8; 1];
    let _ = uart.read(&mut b).await;
}

pub fn init(spawner: &embassy_executor::Spawner, uart: Uart<'static, Async>) {
    spawner.spawn(read_uart(uart).unwrap());
}

pub fn build(spawner: &embassy_executor::Spawner) {
    let p = embassy_stm32::init(embassy_stm32::Config::default());

    let uart = Uart::new(
        p.USART3,
        p.PD9,        // rx
        p.PD8,        // tx
        p.GPDMA1_CH0, // tx dma
        p.GPDMA1_CH1, // rx dma
        Irqs,
        embassy_stm32::usart::Config::default(),
    )
    .unwrap();
    init(spawner, uart);
}
