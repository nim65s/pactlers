//! CDC-ACM serial port example using cortex-m-rtic.
//! Target board: Blue Pill
//! with bincode & rtt
#![no_main]
#![no_std]
#![allow(clippy::unwrap_used)]
#![feature(error_in_core)]

use panic_rtt_target as _;

mod adcs;
mod error;
mod tewma;

#[rtic::app(device = stm32f1xx_hal::pac, peripherals = true, dispatchers = [SPI1, SPI2, SPI3, ADC1_2, ADC3, CAN_RX1, CAN_SCE])]
mod app {
    use bincode::encode_into_slice;
    use cortex_m::asm::delay;
    use pactlers_lib::{Cmd, HEADER, N_ADCS};
    use rtt_target::{rprintln, rtt_init_print};
    use stm32f1xx_hal::adc;
    use stm32f1xx_hal::gpio::PinState;
    use stm32f1xx_hal::gpio::{Output, PushPull, PC13};
    use stm32f1xx_hal::pac::{ADC1, TIM2};
    use stm32f1xx_hal::prelude::*;
    use stm32f1xx_hal::timer::MonoTimerUs;
    use stm32f1xx_hal::usb::{Peripheral, UsbBus, UsbBusType};
    use stm32f1xx_hal::watchdog::IndependentWatchdog;
    use usb_device::prelude::*;

    use crate::adcs::AdcChannel;
    use crate::tewma::Tewmas;

    #[shared]
    struct Shared {
        usb_dev: UsbDevice<'static, UsbBusType>,
        serial: usbd_serial::SerialPort<'static, UsbBusType>,
    }

    #[local]
    struct Local {
        led: PC13<Output<PushPull>>,
        state: bool,
        iwdg: IndependentWatchdog,
        channels: [AdcChannel; N_ADCS],
        adc1: adc::Adc<ADC1>,
        values: Tewmas,
    }

    #[monotonic(binds = TIM2, default = true)]
    type MyMono = MonoTimerUs<TIM2>;

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        rtt_init_print!();
        rprintln!("init start");
        static mut USB_BUS: Option<usb_device::bus::UsbBusAllocator<UsbBusType>> = None;

        let mut flash = cx.device.FLASH.constrain();
        let rcc = cx.device.RCC.constrain();

        let clocks = rcc
            .cfgr
            .use_hse(8.MHz())
            .sysclk(48.MHz())
            .pclk1(24.MHz())
            .freeze(&mut flash.acr);

        assert!(clocks.usbclk_valid());

        let mono = cx.device.TIM2.monotonic_us(&clocks);

        let mut gpioa = cx.device.GPIOA.split();
        let mut gpioc = cx.device.GPIOC.split();

        // BluePill board has a pull-up resistor on the D+ line.
        // Pull the D+ pin down to send a RESET condition to the USB bus.
        // This forced reset is needed only for development, without it host
        // will not reset your device when you upload new firmware.
        let usb_dp = gpioa
            .pa12
            .into_push_pull_output_with_state(&mut gpioa.crh, PinState::Low);
        delay(clocks.sysclk().raw() / 100);

        let usb_dm = gpioa.pa11;
        let usb_dp = usb_dp.into_floating_input(&mut gpioa.crh);

        let usb = Peripheral {
            usb: cx.device.USB,
            pin_dm: usb_dm,
            pin_dp: usb_dp,
        };

        unsafe {
            USB_BUS.replace(UsbBus::new(usb));
        }

        let serial = usbd_serial::SerialPort::new(unsafe { USB_BUS.as_ref().unwrap() });

        let usb_dev = UsbDeviceBuilder::new(
            unsafe { USB_BUS.as_ref().unwrap() },
            UsbVidPid(0x6565, 0x0001),
        )
        .manufacturer("Nim")
        .product("pactlers2")
        .serial_number("0001")
        .device_class(usbd_serial::USB_CLASS_CDC)
        .build();

        let led = gpioc
            .pc13
            .into_push_pull_output_with_state(&mut gpioc.crh, PinState::Low);

        blink::spawn_after(1.secs()).unwrap();
        read::spawn_after(100.millis()).unwrap();

        rprintln!("init end");

        let channels = [
            AdcChannel::A0(gpioa.pa0.into_analog(&mut gpioa.crl)),
            AdcChannel::A1(gpioa.pa1.into_analog(&mut gpioa.crl)),
            AdcChannel::A2(gpioa.pa2.into_analog(&mut gpioa.crl)),
            AdcChannel::A3(gpioa.pa3.into_analog(&mut gpioa.crl)),
            AdcChannel::A4(gpioa.pa4.into_analog(&mut gpioa.crl)),
        ];

        let adc1 = adc::Adc::adc1(cx.device.ADC1, clocks);

        let mut iwdg = IndependentWatchdog::new(cx.device.IWDG);
        iwdg.start(3.secs());

        let values = Tewmas::default();

        (
            Shared { usb_dev, serial },
            Local {
                led,
                state: false,
                iwdg,
                adc1,
                channels,
                values,
            },
            init::Monotonics(mono),
        )
    }

    #[task(binds = USB_HP_CAN_TX, shared = [usb_dev, serial])]
    fn usb_tx(cx: usb_tx::Context) {
        let mut usb_dev = cx.shared.usb_dev;
        let mut serial = cx.shared.serial;

        (&mut usb_dev, &mut serial).lock(|usb_dev, serial| usb_dev.poll(&mut [serial]));
    }

    #[task(binds = USB_LP_CAN_RX0, shared = [usb_dev, serial])]
    fn usb_rx0(cx: usb_rx0::Context) {
        let mut usb_dev = cx.shared.usb_dev;
        let mut serial = cx.shared.serial;

        (&mut usb_dev, &mut serial).lock(|usb_dev, serial| usb_dev.poll(&mut [serial]));
    }

    #[task(local = [led, state, iwdg])]
    fn blink(cx: blink::Context) {
        cx.local.iwdg.feed();
        if *cx.local.state {
            cx.local.led.set_high();
            *cx.local.state = false;
        } else {
            cx.local.led.set_low();
            *cx.local.state = true;
        }

        blink::spawn_after(1.secs()).unwrap();
    }

    #[task(capacity = 5, shared = [serial])]
    fn send(mut cx: send::Context, cmd: Cmd) {
        //rprintln!("send {:?}", cmd);
        let conf = bincode::config::standard();
        let mut buf = [0u8; 32];
        let size = encode_into_slice(cmd, &mut buf, conf).unwrap();
        cx.shared.serial.lock(|serial| {
            serial.write(&HEADER).ok();
            serial.write(&[size.try_into().unwrap()]).ok();
            //rprintln!("encoded {} : {:?}", size, buf);
            serial.write(&buf[0..size]).ok();
        });
    }

    #[task(local = [adc1, channels, values])]
    fn read(cx: read::Context) {
        let adc1 = cx.local.adc1;
        let channels = cx.local.channels;
        let values = cx.local.values;

        for (i, chan) in (0_u8..).zip(channels.iter_mut()) {
            if let Some(cmd) = values.update(i, chan.read(adc1).unwrap()) {
                send::spawn(cmd).unwrap();
            }
        }

        read::spawn_after(100.millis()).unwrap();
    }
}
