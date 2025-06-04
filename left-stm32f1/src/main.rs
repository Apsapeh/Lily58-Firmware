#![no_main]
#![no_std]

use panic_halt as _;

use cortex_m_rt::entry;
use nb::block;
use rtt_target::{rprintln, rtt_init_print};
use stm32f1xx_hal::{pac, prelude::*, serial::Config};

use shared_src::PrimitiveBitset;

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
        /*.pclk1(24.MHz())*/
        .freeze(&mut flash.acr);

    let mut gpioa = dp.GPIOA.split();
    let mut gpiob = dp.GPIOB.split();

    let tx = gpiob.pb10.into_alternate_push_pull(&mut gpiob.crh);
    let rx = gpiob.pb11;

    let mut serial = dp
        .USART3
        .serial((tx, rx), Config::default().baudrate(57600.bps()), &clocks);

    loop {

        // 
        '_receive_loop: loop {
            let received = match serial.rx.read() {
                Ok(r) => r,
                Err(_) => {
                    continue;
                }
            };
            
            if received & 0x80 != 0 {
                let mut packed_data = [0u8; 5];
                packed_data[0] = received;
                for i in 1..5 {
                    packed_data[i] = match block!(serial.rx.read()) {
                        Ok(data) => if data & 0x80 == 0 {data} else {continue '_receive_loop},
                        Err(_) => continue '_receive_loop
                    };
                }

                //rprintln!("{:?}", packed_data);
                let data = unpack_keydata(packed_data);
                rprintln!("data: {:032b}", data);
                //rprintln!("{}", data);
                break;
            }
        }

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