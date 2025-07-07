// USB HID Device Management
// Handles USB device initialization, HID mouse reports, and device state management
// Uses safe mutex-wrapped RefCell pattern instead of unsafe static globals

use core::cell::RefCell;
use critical_section::Mutex;
use rp_pico::hal;
use rp_pico::hal::clocks::UsbClock;
use rp_pico::hal::pac;
use rp_pico::pac::*;

use usb_device::{class_prelude::*, prelude::*};
use usbd_hid::descriptor::generator_prelude::*;
use usbd_hid::descriptor::MouseReport;
use usbd_hid::hid_class::HIDClass;

// Use safer mutex-wrapped RefCell instead of unsafe static mut
static USB_DEVICE: Mutex<RefCell<Option<UsbDevice<hal::usb::UsbBus>>>> = Mutex::new(RefCell::new(None));
static USB_BUS: Mutex<RefCell<Option<UsbBusAllocator<hal::usb::UsbBus>>>> = Mutex::new(RefCell::new(None));
static USB_HID: Mutex<RefCell<Option<HIDClass<hal::usb::UsbBus>>>> = Mutex::new(RefCell::new(None));

pub struct Device {}

impl Device {
    pub fn init(reg: USBCTRL_REGS, dpram: USBCTRL_DPRAM, c: UsbClock, r: &mut RESETS) {
        let usb_bus = UsbBusAllocator::new(hal::usb::UsbBus::new(reg, dpram, c, true, r));

        // Store the bus in the static
        critical_section::with(|cs| {
            USB_BUS.borrow_ref_mut(cs).replace(usb_bus);
        });

        // Get a reference to the bus for creating other USB objects
        let bus_ref = critical_section::with(|cs| {
            USB_BUS.borrow_ref(cs).as_ref().unwrap() as *const UsbBusAllocator<hal::usb::UsbBus>
        });

        // SAFETY: We know the bus is valid and won't be moved
        let bus_ref = unsafe { &*bus_ref };

        let usb_hid = HIDClass::new(bus_ref, MouseReport::desc(), 60);
        critical_section::with(|cs| {
            USB_HID.borrow_ref_mut(cs).replace(usb_hid);
        });

        let usb_dev = UsbDeviceBuilder::new(bus_ref, UsbVidPid(0x16c0, 0x27da))
            .strings(&[StringDescriptors::default()
                .manufacturer("BIG CHUNGUS")
                .product("FOOTPEDAL2")
                .serial_number("053531217003516867")])
            .unwrap()
            .device_class(0)
            .build();
        
        critical_section::with(|cs| {
            USB_DEVICE.borrow_ref_mut(cs).replace(usb_dev);
        });

        unsafe {
            // Enable the USB interrupt
            pac::NVIC::unmask(hal::pac::Interrupt::USBCTRL_IRQ);
        };
    }

    pub fn action(state: bool, button: u8) -> Option<Result<usize, UsbError>> {
        let report = MouseReport {
            x: 0,
            y: 0,
            buttons: (state as u8) << (button - 1),
            wheel: 0,
            pan: 0,
        };

        critical_section::with(|cs| {
            USB_HID.borrow_ref_mut(cs).as_mut().map(|hid| hid.push_input(&report))
        })
    }

    pub fn get_state() -> Option<UsbDeviceState> {
        critical_section::with(|cs| {
            USB_DEVICE.borrow_ref(cs).as_ref().map(|dev| dev.state())
        })
    }
}

#[allow(non_snake_case)]
#[interrupt]
fn USBCTRL_IRQ() {
    critical_section::with(|cs| {
        let mut usb_device = USB_DEVICE.borrow_ref_mut(cs);
        let mut usb_hid = USB_HID.borrow_ref_mut(cs);
        
        if let (Some(usb_dev), Some(hid)) = (usb_device.as_mut(), usb_hid.as_mut()) {
            usb_dev.poll(&mut [hid]);
        }
    });
}
