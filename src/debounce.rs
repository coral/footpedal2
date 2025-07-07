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
    timer: rp_pico::hal::timer::Timer,
    last_change_time: u64,
    debounce_delay: u64,
    stable_state: bool,
    last_input: bool,
}

impl Debouncer {
    // create a new debouncer, debounce_delay is specificed in milliseconds
    pub fn create(
        t: pac::TIMER,
        r: &mut RESETS,
        clocks: &rp_pico::hal::clocks::ClocksManager,
        debounce_delay: u64,
    ) -> Debouncer {
        let timer = rp_pico::hal::timer::Timer::new(t, r, clocks);
        let init = timer.get_counter().ticks();
        Debouncer {
            timer,
            debounce_delay: debounce_delay * 1000,
            last_change_time: init,
            stable_state: false,
            last_input: false,
        }
    }

    pub fn process(&mut self, value: bool) -> DebounceResult {
        let now = self.timer.get_counter().ticks();

        // If input changed, reset the timer
        if value != self.last_input {
            self.last_input = value;
            self.last_change_time = now;
            return DebounceResult::NoChange;
        }

        // If debounce period hasn't elapsed, no change
        if (self.last_change_time + self.debounce_delay) > now {
            return DebounceResult::NoChange;
        }

        // Input has been stable for debounce period - check if state changed
        if value != self.stable_state {
            self.stable_state = value;
            if self.stable_state {
                return DebounceResult::Pressed;
            } else {
                return DebounceResult::Released;
            }
        }

        return DebounceResult::NoChange;
    }
}
