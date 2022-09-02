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
    lc: u64,
    debounce_delay: u64,
    state: bool,
}

impl Debouncer {
    // create a new debouncer, debounce_delay is specificed in milliseconds
    pub fn create(t: pac::TIMER, r: &mut RESETS, debounce_delay: u64) -> Debouncer {
        let timer = rp_pico::hal::timer::Timer::new(t, r);
        let init = timer.get_counter();
        Debouncer {
            timer,
            debounce_delay: debounce_delay * 1000,
            lc: init,
            state: false,
        }
    }

    pub fn process(&mut self, value: bool) -> DebounceResult {
        let mt = self.timer.get_counter();

        if (self.lc + self.debounce_delay) > mt {
            return DebounceResult::NoChange;
        }

        self.lc = mt;

        if value != self.state {
            self.state = value;

            if self.state {
                return DebounceResult::Pressed;
            } else {
                return DebounceResult::Released;
            }
        }

        return DebounceResult::NoChange;
    }
}
