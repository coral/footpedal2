use fugit::{MicrosDurationU32, MicrosDurationU64, TimerInstantU64};

#[repr(u8)]
#[derive(PartialEq)]
pub enum DebounceResult {
    NoChange,
    Pressed,
    Released,
}

pub type Instant = TimerInstantU64<1_000_000>;

fn get_counter(timer: &crate::pac::timer::RegisterBlock) -> Instant {
    let mut hi0 = timer.timerawh.read().bits();
    let timestamp = loop {
        let low = timer.timerawl.read().bits();
        let hi1 = timer.timerawh.read().bits();
        if hi0 == hi1 {
            break (u64::from(hi0) << 32) | u64::from(low);
        }
        hi0 = hi1;
    };
    TimerInstantU64::from_ticks(timestamp)
}

pub struct Debouncer {}

impl Debouncer {
    pub fn create() -> Debouncer {}
}
