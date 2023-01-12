use crate::error::Error;
use stm32f1xx_hal::gpio::{Analog, PA0, PA1, PA2, PA3, PA4, PA5, PA6, PA7, PB0, PB1};
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
    pub fn read(&mut self, adc: &mut adc::Adc<pac::ADC1>) -> Result<u16, Error> {
        match self {
            Self::A0(p) => adc.read(p),
            Self::A1(p) => adc.read(p),
            Self::A2(p) => adc.read(p),
            Self::A3(p) => adc.read(p),
            Self::A4(p) => adc.read(p),
            Self::A5(p) => adc.read(p),
            Self::A6(p) => adc.read(p),
            Self::A7(p) => adc.read(p),
            Self::B0(p) => adc.read(p),
            Self::B1(p) => adc.read(p),
        }
        .map_err(Error::Nb)
    }
}
