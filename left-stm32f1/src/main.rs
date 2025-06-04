#![no_main]
#![no_std]

use panic_halt as _;

use cortex_m_rt::entry;
use cortex_m::asm::delay;
use nb::block;
use rtt_target::{rprintln, rtt_init_print};
use stm32f1xx_hal::{pac, prelude::*, serial::Config};
use stm32f1xx_hal::usb::{Peripheral, UsbBus};
use usb_device::prelude::*;

use usbd_human_interface_device::device::keyboard::{KeyboardLedsReport, NKROBootKeyboardConfig};
use usbd_human_interface_device::page::Keyboard;
use usbd_human_interface_device::prelude::*;

use shared_src::PrimitiveBitset;

mod layouts_def;

#[entry]
fn main() -> ! {
    rtt_init_print!();

    // Get access to the device specific peripherals from the peripheral access crate
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();

    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .sysclk(48.MHz())
        .pclk1(24.MHz())
        .freeze(&mut flash.acr);

    let mut gpioa = dp.GPIOA.split();
    let mut gpiob = dp.GPIOB.split();

    /////// Init UART ///////
    let tx = gpiob.pb10.into_alternate_push_pull(&mut gpiob.crh);
    let rx = gpiob.pb11;
    let mut serial = dp
        .USART3
        .serial((tx, rx), Config::default().baudrate(57600.bps()), &clocks);
    
    /////// Init USB-HID device ///////
    // This code taken from examples
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

    let mut keyboard = UsbHidClassBuilder::new()
        .add_device(NKROBootKeyboardConfig::default())
        .build(&usb_bus);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x05AC, 0x0202))
        .strings(&[StringDescriptors::default()
            .manufacturer("TimofeyKirichenko")
            .product("Potujnaya Keyboard")
            .serial_number("PotujnayaPeremoga")])
        .unwrap()
        .build();
    
    let mut timer = dp.TIM2.counter_hz(&clocks);
    timer.start(1000.Hz()).unwrap();
    
    // Async UART buffer
    let mut uart_buffer = [0u8; 5];
    let mut uart_buffer_len = 0; 

    loop {      
        // Async reading UART data from slave to buffer
        match (serial.rx.read()) {
            Ok(received) => {
                // Message start
                if received & 0x80 == 0x80 {
                    // Buffer is truly start
                    if uart_buffer_len == 0 {
                        uart_buffer[0] = received;
                        uart_buffer_len = 1;
                    } else { // Buffer is corrupted
                        uart_buffer_len = 0;
                    }
                } else {
                    if uart_buffer_len != 0 {
                        uart_buffer[uart_buffer_len as usize] = received;
                        uart_buffer_len += 1;
                    }
                }
            }
            Err(_) => {}
        }
        
        // If buffer filled
        if uart_buffer_len == 5 {
            uart_buffer_len = 0;
            let data = unpack_keydata(uart_buffer);
            let right_matrix = PrimitiveBitset::new(data);
            // TODO: Read left matrix
            
            let mut report = [Keyboard::NoEventIndicated; 2];
            if right_matrix.get(0) {
                report[0] = Keyboard::MediaVolumeUp;
                report[1] = Keyboard::A;
            }
            keyboard.device().write_report(report).ok();
        }
        
        /*let mut report = [Keyboard::NoEventIndicated; 1];
        if right_matrix.get(0) {
            report[0] = Keyboard::A;
        }
        keyboard.device().write_report(report.into_iter()).ok();*/
        
        if timer.wait().is_ok() {
            keyboard.tick().unwrap();
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


/// Byte format:
///  - [0] bit:
///     - 1: It's the message first byte
///     - 0: It's the message continuation byte \[1..5\]
///  -   [1..7] bits: data
fn unpack_keydata(data: [u8; 5]) -> u32 {
    let b0 = (data[0] & 0x7f) as u32;
    let b1 = (data[1] & 0x7f) as u32;
    let b2 = (data[2] & 0x7f) as u32;
    let b3 = (data[3] & 0x7f) as u32;
    let b4 = (data[4] & 0x7f) as u32;

    let le = b0 | (b1 << 7) | (b2 << 14) | (b3 << 21) | (b4 << 28);

    u32::from_le(le)
}
