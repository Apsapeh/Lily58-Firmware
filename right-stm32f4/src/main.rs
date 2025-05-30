#![no_main]
#![no_std]

use core::result;

use panic_halt as _;

use nb::block;

use cortex_m_rt::entry;
use stm32f4xx_hal::{self as hal, serial::Config};

use crate::hal::{pac, prelude::*};


#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    let gpioa = dp.GPIOA.split();

    let rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.use_hse(25.MHz()).freeze();

    let mut delay = dp.TIM1.delay_ms(&clocks);

    // define RX/TX pins
    let tx_pin = gpioa.pa9;
    let rx_pin = gpioa.pa10;

    // configure serial
    // let mut tx = Serial::tx(dp.USART1, tx_pin, 9600.bps(), &clocks).unwrap();
    // or
    let mut serial = dp.USART1.serial::<u8>((tx_pin, rx_pin), Config::default().baudrate(57600.bps()), &clocks).unwrap();
    //let mut tx = dp.USART1.tx(tx_pin, 9600.bps(), &clocks).unwrap();
    //let mut rx = dp.USART1.rx::<u8>(rx_pin, 9600.bps(), &clocks).unwrap();

    let mut value: u8 = 0;

    let gpioc = dp.GPIOC.split();
    let mut led = gpioc.pc13.into_push_pull_output();
    led.set_high();

    let mut key_data: u32 = 0;

    let (mut tx, mut rx) = serial.split();

    loop {
        let packed_keydata =  pack_keydata(key_data);
        for i in 0..5 {
            block!(tx.write(packed_keydata[i])).unwrap();
        }
        led.set_low();
        key_data =key_data.wrapping_add(1);
    }
}

/*
Byte format:
    0 bit:
        1: message first byte
        0: message continuation byte
    1..7 bits: data
*/
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