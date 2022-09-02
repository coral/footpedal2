use embedded_hal::timer;
use fugit::{MicrosDurationU32, MicrosDurationU64, TimerInstantU64};
use rp_pico::pac;

#[repr(u8)]
#[derive(PartialEq)]
pub enum DebounceResult {
    NoChange,
    Pressed,
    Released,
}

pub struct Debouncer {
    t: rp2040_hal::timer::Timer,

    release: u64,
}

impl Debouncer {
    pub fn create(t: pac::TIMER) -> Debouncer {
        let t = rp_pico::hal::timer::Timer::new(t, pac::RESETS);

        Debouncer { t, release: 100 }
    }

    pub fn debounce(value: bool) -> bool {
        return false;
    }
}
