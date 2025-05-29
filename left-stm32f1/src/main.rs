//! CDC-ACM serial port example using polling in a busy loop.
//! Target board: Blue Pill
//!
//! Note:
//! When building this since this is a larger program,
//! one would need to build it using release profile
//! since debug profiles generates artifacts that
//! cause FLASH overflow errors due to their size
#![no_std]
#![no_main]

//extern crate panic_semihosting;

use cortex_m::asm::delay;
use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32f1xx_hal::usb::{Peripheral, UsbBus};
use stm32f1xx_hal::{pac, prelude::*};
use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

use usbd_human_interface_device::page::Keyboard;
use usbd_human_interface_device::device::keyboard::{KeyboardLedsReport, NKROBootKeyboardConfig};
use usbd_human_interface_device::prelude::*;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Hello, world!");
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();

    rprintln!("Timer started");


    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .sysclk(48.MHz())
        .pclk1(24.MHz())
        .freeze(&mut flash.acr);

    assert!(clocks.usbclk_valid());

    // Configure the on-board LED (PC13, green)
    let mut gpioc = dp.GPIOC.split();
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    led.set_high(); // Turn off

    let mut gpioa = dp.GPIOA.split();

    // BluePill board has a pull-up resistor on the D+ line.
    // Pull the D+ pin down to send a RESET condition to the USB bus.
    // This forced reset is needed only for development, without it host
    // will not reset your device when you upload new firmware.
    let mut usb_dp = gpioa.pa12.into_push_pull_output(&mut gpioa.crh);
    usb_dp.set_low();
    delay(clocks.sysclk().raw() / 100);

    let usb = Peripheral {
        usb: dp.USB,
        pin_dm: gpioa.pa11,
        pin_dp: usb_dp.into_floating_input(&mut gpioa.crh),
    };
    let usb_bus = UsbBus::new(usb);

    let mut serial = SerialPort::new(&usb_bus);

    let mut keyboard = UsbHidClassBuilder::new()
    .add_device(
        NKROBootKeyboardConfig::default(),
    )
    .build(&usb_bus);

let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x1209, 0x0001))
    .strings(&[
        StringDescriptors::default()
            .manufacturer("usbd-human-interface-device")
            .product("NKRO Keyboard")
            .serial_number("TEST")]
    ).unwrap()       
    .build();

    let mut timer = dp.TIM2.counter_hz(&clocks);
    let mut timer2 = dp.TIM3.counter_hz(&clocks);
    timer.start(1000.Hz()).unwrap();
    timer2.start(1.Hz()).unwrap();

    let mut report = [Keyboard::NoEventIndicated; 1];
    let mut hold = false;
    loop {
        if timer2.wait().is_ok() {
            led.toggle();

            hold = !hold;
            if hold {
                report[0] = Keyboard::W;
            } else {
                report[0] = Keyboard::NoEventIndicated;
            }
        } else {
        }
        keyboard.device().write_report(report).ok();
        

    // tick once per ms/at 1kHz
    if timer.wait().is_ok() {
        keyboard.tick().unwrap();
        //timer.start(1.kHz()).unwrap();
    } else {
    }

    if usb_dev.poll(&mut [&mut keyboard]) {
        match keyboard.device().read_report() {

            Ok(l) => {
                //update_leds(l);
            }
            _ => {}

        }
    }
    }
}