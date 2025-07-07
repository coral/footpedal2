// LUL

#![no_std]
#![no_main]

use embedded_hal::digital::{InputPin, OutputPin};
use panic_reset as _;
use rp_pico::entry;
use rp_pico::hal;
use rp_pico::hal::pac;
use usb_device::device::UsbDeviceState;

mod debounce;
mod device;

const MOUSE_BUTTON: u8 = 8;              // Which mouse button to simulate (1-8)
const DEBOUNCE_DELAY_MS: u64 = 10;       // Button debounce delay in milliseconds
const USB_RETRY_LIMIT: u8 = 3;           // Number of USB operation retries
const USB_RETRY_DELAY_MS: u32 = 100;     // Delay between USB retries in milliseconds

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

    //Initalize debouncer
    let mut deb = debounce::Debouncer::create(pac.TIMER, &mut pac.RESETS, &clocks, DEBOUNCE_DELAY_MS);

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
    // button input - using configurable pin (currently hardcoded to GPIO19)
    let mut button_pin = pins.gpio19.into_pull_up_input(); // TODO: Make this configurable based on BUTTON_PIN
    
    // Track previous USB state to detect changes
    let mut previous_usb_state = UsbDeviceState::Default;

    loop {
        // Feed the watchdog to prevent reset
        watchdog.feed();
        
        // Monitor USB connection state
        let current_usb_state = device::Device::get_state().unwrap_or(UsbDeviceState::Default);
        if current_usb_state != previous_usb_state {
            // USB state changed - this could indicate host sleep/wake
            // Flash LED to indicate state change
            match current_usb_state {
                UsbDeviceState::Configured => {
                    // Host is awake and device is configured
                    led_pin.set_high().unwrap();
                }
                UsbDeviceState::Suspend => {
                    // Host went to sleep
                    led_pin.set_low().unwrap();
                }
                _ => {
                    // Other states (Default, Addressed) - brief flash
                    led_pin.set_high().unwrap();
                }
            }
            previous_usb_state = current_usb_state;
        }
        
        let res = match deb.process(button_pin.is_high().unwrap()) {
            debounce::DebounceResult::NoChange => {
                continue;
            }
            debounce::DebounceResult::Pressed => {
                // Only send button press if USB is configured
                if current_usb_state == UsbDeviceState::Configured {
                    led_pin.set_high().unwrap();
                    send_usb_action_with_retry(true, MOUSE_BUTTON, &mut watchdog)
                } else {
                    // USB not ready - skip action
                    continue;
                }
            }
            debounce::DebounceResult::Released => {
                // Only send button release if USB is configured
                if current_usb_state == UsbDeviceState::Configured {
                    led_pin.set_low().unwrap();
                    send_usb_action_with_retry(false, MOUSE_BUTTON, &mut watchdog)
                } else {
                    // USB not ready - skip action
                    continue;
                }
            }
        };

        // Process the result with improved error handling
        if let Some(result) = res {
            match result {
                Ok(_) => {
                    // Success - continue normally
                }
                Err(e) => {
                    handle_usb_error(e);
                }
            }
        }
    }
}

fn send_usb_action_with_retry(state: bool, button: u8, watchdog: &mut hal::Watchdog) -> Option<Result<usize, usb_device::UsbError>> {
    for _attempt in 0..USB_RETRY_LIMIT {
        // Feed watchdog during retry attempts
        watchdog.feed();
        
        let result = device::Device::action(state, button);
        
        if let Some(Ok(_)) = result {
            return result; // Success
        }
        
        if let Some(Err(e)) = result {
            match e {
                usb_device::UsbError::WouldBlock => {
                    // This is expected during busy periods - retry
                    simple_delay_ms(USB_RETRY_DELAY_MS);
                    continue;
                }
                usb_device::UsbError::BufferOverflow => {
                    // Buffer full - wait and retry
                    simple_delay_ms(USB_RETRY_DELAY_MS);
                    continue;
                }
                _ => {
                    // Other errors - return immediately
                    return result;
                }
            }
        }
    }
    
    // All retries exhausted - return last attempt result
    device::Device::action(state, button)
}

fn handle_usb_error(error: usb_device::UsbError) {
    match error {
        usb_device::UsbError::WouldBlock => {
            // This should have been handled by retry logic
        }
        usb_device::UsbError::ParseError | 
        usb_device::UsbError::InvalidState |
        usb_device::UsbError::BufferOverflow => {
            // These are common during host sleep/wake transitions - continue
        }
        usb_device::UsbError::EndpointOverflow |
        usb_device::UsbError::EndpointMemoryOverflow |
        usb_device::UsbError::InvalidEndpoint => {
            // These indicate more serious issues but may be recoverable
        }
        usb_device::UsbError::Unsupported => {
            // This is likely a permanent error - reset as last resort
            cortex_m::peripheral::SCB::sys_reset();
        }
    }
}

fn simple_delay_ms(ms: u32) {
    // Simple delay using busy wait
    // In a real implementation, you might want to use a timer-based delay
    for _ in 0..(ms * 1000) {
        cortex_m::asm::nop();
    }
}
