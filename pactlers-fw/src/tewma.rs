use pactlers_lib::{Cmd, N_ADCS};
const THRESHOLD: u16 = 10;
//const N: u16 = 3;
//const ALPHA: f32 = 2. / (N as f32 + 1.);
const ALPHA: usize = 1; // "floor(x * 2 / (3 + 1))" == "x >> 1""

type AdcValues = [i16; N_ADCS];

// Thresholded Exponentially Weighted Moving Average
pub struct Tewmas {
    ewma: AdcValues,
    tewma: AdcValues, // reduce bandwdidh
}

impl Tewmas {
    pub fn new() -> Tewmas {
        Tewmas {
            ewma: [0; N_ADCS],
            tewma: [0; N_ADCS],
        }
    }

    pub fn update(&mut self, i: usize, val: i16) -> bool {
        //self.ewma[i] = (self.ewma[i] as f32 + ALPHA * (val - self.ewma[i]) as f32) as i32;
        self.ewma[i] += (val - self.ewma[i]) >> ALPHA;
        if self.tewma[i].abs_diff(self.ewma[i]) > THRESHOLD {
            self.tewma[i] = self.ewma[i];
            true
        } else {
            false
        }
    }

    pub fn get(&self, i: usize) -> Cmd {
        Cmd { select: i as u8, volume: self.tewma[i] as u16 }
    }
}
