//! Serial interface loopback test
//!
//! You have to short the TX and RX pins to make this program work

#![allow(clippy::empty_loop)]
#![deny(unsafe_code)]
#![no_main]
#![no_std]

use panic_halt as _;


use cortex_m_rt::entry;
use rtt_target::{rprintln, rtt_init_print};
use stm32f1xx_hal::{pac, prelude::*, serial::Config};

use usbd_serial::embedded_io::Write;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Hello, world!");
    // Get access to the device specific peripherals from the peripheral access crate
    let p = pac::Peripherals::take().unwrap();

    // Take ownership over the raw flash and rcc devices and convert them into the corresponding
    // HAL structs
    let mut flash = p.FLASH.constrain();
    let rcc = p.RCC.constrain();

    // Freeze the configuration of all the clocks in the system and store the frozen frequencies in
    // `clocks`
    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .sysclk(48.MHz())
        /*.pclk1(24.MHz())*/
        .freeze(&mut flash.acr);

    // Prepare the alternate function I/O registers
    //let mut afio = p.AFIO.constrain();

    // Prepare the GPIOB peripheral
    let mut gpioa = p.GPIOA.split();

    // USART1
    let tx = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
    let rx = gpioa.pa10;

    // USART1
    // let tx = gpiob.pb6.into_alternate_push_pull(&mut gpiob.crl);
    // let rx = gpiob.pb7;

    // USART2
    // let tx = gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl);
    // let rx = gpioa.pa3;

    // USART3
    // Configure pb10 as a push_pull output, this will be the tx pin
    // Take ownership over pb11

    // Set up the usart device. Take ownership over the USART register and tx/rx pins. The rest of
    // the registers are used to enable and configure the device.
    let mut serial = p
        .USART1
        .serial((tx, rx), Config::default().baudrate(9600.bps()), &clocks);

    /*// Loopback test. Write `X` and wait until the write is successful.
    let sent = b'X';
    block!(serial.tx.write_u8(sent)).unwrap();

    // Read the byte that was just sent. Blocks until the read is complete
    let received = block!(serial.rx.read()).unwrap();

    // Since we have connected tx and rx, the byte we sent should be the one we received
    assert_eq!(received, sent);

    // Trigger a breakpoint to allow us to inspect the values
    asm::bkpt();

    // You can also split the serial struct into a receiving and a transmitting part
    let (mut tx, mut rx) = serial.split();
    let received = block!(rx.read()).unwrap();
    //let sent = b'Y';
    block!(tx.write_u8(received)).unwrap();
    //assert_eq!(received, sent);
    asm::bkpt();*/

    // read rx buffer in loop

    let mut gpioc = p.GPIOC.split();
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    led.set_low();
    rprintln!("A");
    let cp = cortex_m::Peripherals::take().unwrap();
    //let mut timer = Timer::
    //timer.start(1.Hz()).unwrap();
    rprintln!("A");

    loop {
        serial.tx.write_all(&[10u8]);
        /*loop {

            //rprintln!("Wait");
            let received = match serial.rx.read() {
                Ok(r) => r,
                Err(e) => {
                    rprintln!("{:?}", e);
                    continue;
                }
            };
            rprintln!("received: {}", received);
            if received & 0x80 != 0 {
                let mut packed_data = [0u8; 5];
                packed_data[0] = received;
                for i in 1..5 {
                    let received = block!(serial.rx.read()).unwrap();
                    packed_data[i] = received;
                }

                let data = unpack_keydata(packed_data);
                rprintln!("data: {}", data);
                break;
            }
        }*/

        //block!(timer.wait()).unwrap();

        led.toggle();
    }
}

fn unpack_keydata(data: [u8; 5]) -> u32 {
    let b0 = (data[0] & 0x7f) as u32;
    let b1 = (data[1] & 0x7f) as u32;
    let b2 = (data[2] & 0x7f) as u32;
    let b3 = (data[3] & 0x7f) as u32;
    let b4 = (data[4] & 0x7f) as u32;

    let le = b0 | (b1 << 7) | (b2 << 14) | (b3 << 21) | (b4 << 28);

    u32::from_le(le)
}
