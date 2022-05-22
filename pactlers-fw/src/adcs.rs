use stm32f1xx_hal::gpio::*;
use stm32f1xx_hal::{adc, pac, prelude::*};

#[allow(dead_code)]
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
    pub fn read(&mut self, adc: &mut adc::Adc<pac::ADC1>) -> u16 {
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
