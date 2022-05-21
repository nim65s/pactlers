#![no_std]
#![no_main]

use core::panic::PanicInfo;
use cortex_m::asm::delay;
use rtt_target::{rprintln, rtt_init_print};
use stm32f1xx_hal::gpio::*;
use stm32f1xx_hal::usb::{Peripheral, UsbBus};
use stm32f1xx_hal::{adc, pac, prelude::*};
use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

const N_ADCS: usize = 5;

pub enum AdcChannel {
    A0(PA0<Analog>),
    A1(PA1<Analog>),
    A2(PA2<Analog>),
    A3(PA3<Analog>),
    A4(PA4<Analog>),
    A5(PA5<Analog>),
    A6(PA6<Analog>),
    A7(PA7<Analog>),
    B0(PB0<Analog>),
    B1(PB1<Analog>),
}

impl AdcChannel {
    fn read(&mut self, adc: &mut adc::Adc<pac::ADC1>) -> u16 {
        match self {
            AdcChannel::A0(p) => adc.read(p).unwrap(),
            AdcChannel::A1(p) => adc.read(p).unwrap(),
            AdcChannel::A2(p) => adc.read(p).unwrap(),
            AdcChannel::A3(p) => adc.read(p).unwrap(),
            AdcChannel::A4(p) => adc.read(p).unwrap(),
            AdcChannel::A5(p) => adc.read(p).unwrap(),
            AdcChannel::A6(p) => adc.read(p).unwrap(),
            AdcChannel::A7(p) => adc.read(p).unwrap(),
            AdcChannel::B0(p) => adc.read(p).unwrap(),
            AdcChannel::B1(p) => adc.read(p).unwrap(),
        }
    }
}

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

    let mut buf = [0u8; 2 * N_ADCS];
    loop {
        if !usb_dev.poll(&mut [&mut serial]) {
            continue;
        }

        for i in 0..N_ADCS {
            [buf[2 * i], buf[2 * i + 1]] = channels[i].read(&mut adc1).to_le_bytes();
        }

        match serial.write(&buf) {
            Ok(count) if count != 2 * N_ADCS => {
                rprintln!("warning: {} byte written", count);
            }
            _ => {}
        }
    }
}

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rprintln!("{}", info);
    loop {} // You might need a compiler fence in here.
}
