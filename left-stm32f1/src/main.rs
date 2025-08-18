#![no_main]
#![no_std]

use panic_reset as _;

use cortex_m::asm::delay;
use cortex_m_rt::entry;
use stm32f1xx_hal::usb::{Peripheral, UsbBus};
use stm32f1xx_hal::{pac, prelude::*, serial::Config};
use usb_device::prelude::*;

use usbd_human_interface_device::device::consumer::{
    ConsumerControl, ConsumerControlConfig, MultipleConsumerReport,
};
use usbd_human_interface_device::device::keyboard::{NKROBootKeyboard, NKROBootKeyboardConfig};

use usbd_human_interface_device::page::{Consumer, Keyboard};
use usbd_human_interface_device::prelude::*;

use fixed_vec::FixedVec;
use shared_src::PrimitiveBitset;

mod fixed_vec;
mod layouts_def;

#[entry]
fn main() -> ! {
    // Get access to the device specific peripherals from the peripheral access crate
    let dp = pac::Peripherals::take().unwrap_or_else(|| panic!());

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
    // This code taken from the examples
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
        .add_device(ConsumerControlConfig::default())
        .build(&usb_bus);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x05AC, 0x0202))
        .strings(&[StringDescriptors::default()
            .manufacturer("MegaHoholTimofeyKirichenko")
            .product("VirhPotujnosti")
            .serial_number("PesPatron")])
        .unwrap_or_else(|_| panic!())
        .build();

    let mut timer = dp.TIM2.counter_hz(&clocks);
    timer.start(1000.Hz()).unwrap_or_else(|_| panic!());

    //
    // Collumns
    let mut power_pins = [
        gpiob.pb9.into_push_pull_output(&mut gpiob.crh).erase(),
        gpiob.pb8.into_push_pull_output(&mut gpiob.crh).erase(),
        gpiob.pb7.into_push_pull_output(&mut gpiob.crl).erase(),
        gpiob.pb6.into_push_pull_output(&mut gpiob.crl).erase(),
        gpiob.pb5.into_push_pull_output(&mut gpiob.crl).erase(),
    ];

    // Rows
    let signal_pins = [
        gpioa.pa1.into_pull_down_input(&mut gpioa.crl).erase(),
        gpioa.pa2.into_pull_down_input(&mut gpioa.crl).erase(),
        gpioa.pa3.into_pull_down_input(&mut gpioa.crl).erase(),
        gpioa.pa4.into_pull_down_input(&mut gpioa.crl).erase(),
        gpioa.pa5.into_pull_down_input(&mut gpioa.crl).erase(),
        gpioa.pa6.into_pull_down_input(&mut gpioa.crl).erase(),
    ];

    // Async UART buffer
    let mut uart_buffer = [0u8; 5];
    let mut uart_buffer_len = 0;

    let mut left_matrix = [false; 30];
    let mut key_report: FixedVec<_, 58> = FixedVec::new(Keyboard::NoEventIndicated);
    let mut media_report: FixedVec<_, 4> = FixedVec::new(Consumer::Unassigned);

    loop {
        // Async reading UART data from slave to buffer
        match serial.rx.read() {
            Ok(received) => {
                // Message start
                if received & 0x80 == 0x80 {
                    // Buffer is truly start
                    if uart_buffer_len == 0 {
                        uart_buffer[0] = received;
                        uart_buffer_len = 1;
                    } else {
                        // Buffer is corrupted
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

            // Read left matrix
            for (r, pw) in power_pins.iter_mut().enumerate() {
                pw.set_high();
                delay(10);
                for (c, sg) in signal_pins.iter().enumerate() {
                    left_matrix[r * signal_pins.len() + c] = sg.is_high();
                }
                pw.set_low();
            }

            layouts_def::get_report(
                &left_matrix,
                right_matrix,
                &mut key_report,
                &mut media_report,
            );

            keyboard
                .device::<NKROBootKeyboard<'_, _>, _>()
                .write_report(key_report.data)
                .ok();
            keyboard
                .device::<ConsumerControl<'_, _>, _>()
                .write_report(&MultipleConsumerReport {
                    codes: media_report.data,
                })
                .ok();
        }

        if timer.wait().is_ok() {
            keyboard.tick().unwrap_or_else(|_| panic!());
        }

        if usb_dev.poll(&mut [&mut keyboard]) {
            let _ = keyboard.device::<NKROBootKeyboard<'_, _>, _>().read_report();
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
