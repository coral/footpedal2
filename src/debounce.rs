use embedded_hal::timer;
use fugit::{MicrosDurationU32, MicrosDurationU64, TimerInstantU64};
use rp_pico::pac;
use rp_pico::pac::*;

#[repr(u8)]
#[derive(PartialEq)]
pub enum DebounceResult {
    NoChange,
    Pressed,
    Released,
}

pub struct Debouncer {
    timer: rp2040_hal::timer::Timer,
    release: u64,
}

impl Debouncer {
    pub fn create(t: pac::TIMER, r: &mut RESETS, release: u64) -> Debouncer {
        Debouncer {
            timer: rp_pico::hal::timer::Timer::new(t, r),
            release,
        }
    }

    pub fn debounce(value: bool) -> bool {
        return false;
    }
}
