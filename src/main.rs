#![no_std]
#![no_main]

use embedded_hal::digital::v2::{InputPin, OutputPin};
use panic_halt as _;
use rp_pico::entry;
use rp_pico::hal;
use rp_pico::hal::pac;

mod debounce;
mod device;

// Define the footpedal mouse button
const MOUSE_BUTTON: u8 = 8;

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();

    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);
    let clocks = hal::clocks::init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    device::Device::init(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        &mut pac.RESETS,
    );

    let sio = hal::Sio::new(pac.SIO);

    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // onboard led
    let mut led_pin = pins.led.into_push_pull_output();
    // button input
    let button_pin = pins.gpio19.into_pull_up_input();

    //Initalize debouncer
    let mut deb = debounce::Debouncer::create(pac.TIMER, &mut pac.RESETS, 1000);

    loop {
        match deb.process(button_pin.is_high().unwrap()) {
            debounce::DebounceResult::NoChange => {}
            debounce::DebounceResult::Pressed => {
                led_pin.set_high().unwrap();
                device::Device::action(true, MOUSE_BUTTON);
            }
            debounce::DebounceResult::Released => {
                led_pin.set_low().unwrap();
                device::Device::action(false, MOUSE_BUTTON);
            }
        }
        // if button_pin.is_high().unwrap() {
        //     led_pin.set_high().unwrap();
        //     device::Device::action(true, MOUSE_BUTTON);
        // } else {
        //     led_pin.set_low().unwrap();
        //     device::Device::action(false, MOUSE_BUTTON);
        // }
    }
}
