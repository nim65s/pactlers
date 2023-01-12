use pactlers_lib::{Cmd, N_ADCS};
const THRESHOLD: u16 = 10;
//const N: u16 = 3;
//const ALPHA: f32 = 2. / (N as f32 + 1.);
//const ALPHA: usize = 1; // "floor(x * 2 / (3 + 1))" == "x >> 1""

type AdcValues = [u16; N_ADCS];

// Thresholded Exponentially Weighted Moving Average
pub struct Tewmas {
    ewma: AdcValues,
    tewma: AdcValues, // reduce bandwdidh
}

impl Tewmas {
    pub fn update(&mut self, idx: u8, val: u16) -> Option<Cmd> {
        //self.ewma[i] = (self.ewma[i] as f32 + ALPHA * (val - self.ewma[i]) as f32) as i32;
        let i = idx as usize;
        self.ewma[i] = (val + self.ewma[i]) >> 1;
        if self.tewma[i].abs_diff(self.ewma[i]) > THRESHOLD {
            self.tewma[i] = self.ewma[i];
            Some(Cmd::new(idx, self.tewma[i]))
        } else {
            None
        }
    }
}

impl Default for Tewmas {
    fn default() -> Self {
        Self {
            ewma: [0; N_ADCS],
            tewma: [0; N_ADCS],
        }
    }
}
