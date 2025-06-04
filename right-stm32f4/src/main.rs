#![no_main]
#![no_std]
use panic_halt as _;

use cortex_m_rt::entry;
use nb::block;
use stm32f4xx_hal::{self as hal};
use crate::hal::{pac, prelude::*};

use shared_src::PrimitiveBitset;


#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.use_hse(25.MHz()).freeze();

    let gpioa = dp.GPIOA.split();
    let gpiob = dp.GPIOB.split();

    let tx_pin = gpioa.pa2;
    let mut tx = dp.USART2.tx(tx_pin, 57600.bps(), &clocks).unwrap();

    // Collumns
    let mut power_pins = [
        gpiob.pb9.into_push_pull_output().erase(),
        gpiob.pb8.into_push_pull_output().erase(),
        gpiob.pb7.into_push_pull_output().erase(),
        gpiob.pb6.into_push_pull_output().erase(),
        gpiob.pb5.into_push_pull_output().erase(),
    ];

    // Rows
    let signal_pins = [
        gpiob.pb10.into_pull_down_input().erase(),
        gpioa.pa5.into_pull_down_input().erase(),
        gpioa.pa6.into_pull_down_input().erase(),
        gpioa.pa7.into_pull_down_input().erase(),
        gpiob.pb0.into_pull_down_input().erase(),
        gpiob.pb1.into_pull_down_input().erase(),
    ];

    let mut matrix = PrimitiveBitset::new(0u32);

    let mut delay = dp.TIM1.delay_us(&clocks);
    loop {
        delay.delay_us(50);
        // Read keyboard matrix
        for (r, pw) in power_pins.iter_mut().enumerate() {
            pw.set_high();
            delay.delay_us(10);
            for (c, sg) in signal_pins.iter().enumerate() {
                matrix.set(r * signal_pins.len() + c, sg.is_high());
            }
            pw.set_low();
        }
        

        // Send data to the main half (left stm32f1)
        let packed_keydata = pack_keydata(matrix.get_raw());
        for i in 0..5 {
            let _ = block!(tx.write(packed_keydata[i]));
        }
    }
}



/// Byte format:
///  - [0] bit:
///     - 1: It's the message first byte
///     - 0: It's the message continuation byte \[1..5\]
///  -   [1..7] bits: data
fn pack_keydata(key_data: u32) -> [u8; 5] {
    let le = key_data.to_le();
    let mut result = [0; 5];
    result[0] = le as u8 & 0x7f | 0x80;
    result[1] = (le >> 7) as u8 & 0x7f;
    result[2] = (le >> 14) as u8 & 0x7f;
    result[3] = (le >> 21) as u8 & 0x7f;
    result[4] = (le >> 28) as u8 & 0x7f;
    result
}