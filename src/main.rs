#![no_std]
#![no_main]

use core::panic::PanicInfo;
use rtt_target::{rprintln, rtt_init_print};
use stm32f1xx_hal::{adc, pac, prelude::*};

#[cortex_m_rt::entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Hello");

    let p = pac::Peripherals::take().unwrap();
    let mut flash = p.FLASH.constrain();
    let rcc = p.RCC.constrain();

    // Configure ADC clocks
    // Default value is the slowest possible ADC clock: PCLK2 / 8. Meanwhile ADC
    // clock is configurable. So its frequency may be tweaked to meet certain
    // practical needs. User specified value is be approximated using supported
    // prescaler values 2/4/6/8.
    let clocks = rcc.cfgr.adcclk(2.MHz()).freeze(&mut flash.acr);

    // Setup ADC
    let mut adc1 = adc::Adc::adc1(p.ADC1, clocks);

    // Setup GPIOA
    let mut gpioa = p.GPIOA.split();

    // Configure pb0, pb1 as an analog input
    let mut ch0 = gpioa.pa0.into_analog(&mut gpioa.crl);

    loop {
        let data: u16 = adc1.read(&mut ch0).unwrap();
        rprintln!("adc1: {}", data);
    }
}

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rprintln!("{}", info);
    loop {} // You might need a compiler fence in here.
}
