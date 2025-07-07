use rp_pico::hal;
use rp_pico::hal::clocks::UsbClock;
use rp_pico::hal::pac;
use rp_pico::pac::*;

use usb_device::{class_prelude::*, prelude::*};
use usbd_hid::descriptor::generator_prelude::*;
use usbd_hid::descriptor::MouseReport;
use usbd_hid::hid_class::HIDClass;

static mut USB_DEVICE: Option<UsbDevice<hal::usb::UsbBus>> = None;
static mut USB_BUS: Option<UsbBusAllocator<hal::usb::UsbBus>> = None;
static mut USB_HID: Option<HIDClass<hal::usb::UsbBus>> = None;

pub struct Device {}

impl Device {
    pub fn init(reg: USBCTRL_REGS, dpram: USBCTRL_DPRAM, c: UsbClock, r: &mut RESETS) {
        let usb_bus = UsbBusAllocator::new(hal::usb::UsbBus::new(reg, dpram, c, true, r));

        unsafe {
            // Note (safety): This is safe as interrupts haven't been started yet
            USB_BUS = Some(usb_bus);
        }

        let bus_ref = unsafe { USB_BUS.as_ref().unwrap() };

        let usb_hid = HIDClass::new(bus_ref, MouseReport::desc(), 60);
        unsafe {
            // Note (safety): This is safe as interrupts haven't been started yet.
            USB_HID = Some(usb_hid);
        }

        let usb_dev = UsbDeviceBuilder::new(bus_ref, UsbVidPid(0x16c0, 0x27da))
            .strings(&[StringDescriptors::default()
                .manufacturer("BIG CHUNGUS")
                .product("FOOTPEDAL2")
                .serial_number("053531217003516867")])
            .unwrap()
            .device_class(0)
            .build();
        unsafe {
            // Note (safety): This is safe as interrupts haven't been started yet
            USB_DEVICE = Some(usb_dev);
        }

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

        critical_section::with(|_| unsafe { USB_HID.as_mut().map(|hid| hid.push_input(&report)) })
    }
}

#[allow(non_snake_case)]
#[interrupt]
unsafe fn USBCTRL_IRQ() {
    // Handle USB request
    if let (Some(usb_dev), Some(hid)) = (USB_DEVICE.as_mut(), USB_HID.as_mut()) {
        usb_dev.poll(&mut [hid]);
    }
}
