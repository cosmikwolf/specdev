//! `task` — `#[embassy_executor::task]` free-fn emission + the init spawn site.
//! Mirrors the README's task + "Spawning at init" examples (the `.unwrap()` goes
//! on the task call, per embassy-executor 0.10+).
use embassy_executor::Spawner;

#[embassy_executor::task]
async fn read_sensor() {
    loop {
        embassy_time::Timer::after_millis(1).await;
    }
}

#[embassy_executor::task(pool_size = 4)]
async fn worker(_id: u8) {}

pub fn init(spawner: &Spawner) {
    spawner.spawn(read_sensor().unwrap());
    spawner.spawn(worker(0).unwrap());
}
