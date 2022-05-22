#![no_std]
#![no_main]

use core::panic::PanicInfo;
use cortex_m::asm::delay;
use rtt_target::{rprintln, rtt_init_print};
use stm32f1xx_hal::usb::{Peripheral, UsbBus};
use stm32f1xx_hal::{adc, pac, prelude::*};
use usb_device::prelude::{UsbDeviceBuilder, UsbVidPid};
use usbd_serial::{SerialPort, USB_CLASS_CDC};

mod adcs;
mod tewma;
use crate::adcs::AdcChannel;
use crate::tewma::Tewmas;

const N_ADCS: usize = 5;

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
    let clocks = rcc
        .cfgr
        .adcclk(2.MHz())
        .use_hse(8.MHz())
        .sysclk(48.MHz())
        .pclk1(24.MHz())
        .freeze(&mut flash.acr);

    assert!(clocks.usbclk_valid());

    // Setup ADC
    let mut adc1 = adc::Adc::adc1(p.ADC1, clocks);

    // Setup GPIOA
    let mut gpioa = p.GPIOA.split();

    let mut channels: [AdcChannel; N_ADCS] = [
        AdcChannel::A0(gpioa.pa0.into_analog(&mut gpioa.crl)),
        AdcChannel::A1(gpioa.pa1.into_analog(&mut gpioa.crl)),
        AdcChannel::A2(gpioa.pa2.into_analog(&mut gpioa.crl)),
        AdcChannel::A3(gpioa.pa3.into_analog(&mut gpioa.crl)),
        AdcChannel::A4(gpioa.pa4.into_analog(&mut gpioa.crl)),
    ];

    // BluePill board has a pull-up resistor on the D+ line.
    // Pull the D+ pin down to send a RESET condition to the USB bus.
    // This forced reset is needed only for development, without it host
    // will not reset your device when you upload new firmware.
    let mut usb_dp = gpioa.pa12.into_push_pull_output(&mut gpioa.crh);
    usb_dp.set_low();
    delay(clocks.sysclk().raw() / 100);

    let usb = Peripheral {
        usb: p.USB,
        pin_dm: gpioa.pa11,
        pin_dp: usb_dp.into_floating_input(&mut gpioa.crh),
    };
    let usb_bus = UsbBus::new(usb);

    let mut serial = SerialPort::new(&usb_bus);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x6565, 0x0001))
        .manufacturer("Nim")
        .product("pactlers-fw")
        .serial_number("0001")
        .device_class(USB_CLASS_CDC)
        .build();

    let mut values = Tewmas::new();
    loop {
        for (i, chan) in channels.iter_mut().enumerate() {
            usb_dev.poll(&mut [&mut serial]);
            if values.update(i, chan.read(&mut adc1).try_into().unwrap()) {
                usb_dev.poll(&mut [&mut serial]);
                //rprintln!("{} {}", i, values.tewma[i]);
                match serial.write(&values.get(i)) {
                    Ok(count) if count != 3 => {
                        rprintln!("warning: {} byte written", count);
                    }
                    _ => {}
                }
            }
        }
    }
}

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rprintln!("{}", info);
    loop {} // You might need a compiler fence in here.
}
