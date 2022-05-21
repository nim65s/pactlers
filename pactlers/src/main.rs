use std::time::Duration;

const N_ADCS: usize = 5;
const THRESHOLD: u16 = 10;
//const N: u16 = 3;
//const ALPHA: f32 = 2. / (N as f32 + 1.);
const ALPHA: usize = 1; // "floor(x * 2 / (3 + 1))" == "x >> 1""

type AdcValues = [i16; N_ADCS];
type AdcData = [u8; N_ADCS * 2];

fn unpack(data: AdcData) -> AdcValues {
    [
        i16::from_le_bytes([data[8], data[9]]),
        i16::from_le_bytes([data[6], data[7]]),
        i16::from_le_bytes([data[4], data[5]]),
        i16::from_le_bytes([data[2], data[3]]),
        i16::from_le_bytes([data[0], data[1]]),
    ]
}

// Thresholded Exponentially Weighted Moving Average
struct Tewmas {
    ewma: AdcValues,
    tewma: AdcValues, // reduce bandwdidh
}

impl Tewmas {
    pub fn new(data: AdcData) -> Tewmas {
        let vals = unpack(data);
        Tewmas {
            ewma: vals,
            tewma: vals,
        }
    }

    pub fn update(&mut self, data: AdcData) -> bool {
        let mut something_new = false;
        for (i, val) in unpack(data).iter().enumerate() {
            //self.ewma[i] = (self.ewma[i] as f32 + ALPHA * (val - self.ewma[i]) as f32) as i32;
            self.ewma[i] += (val - self.ewma[i]) >> ALPHA;
            if self.tewma[i].abs_diff(self.ewma[i]) > THRESHOLD {
                self.tewma[i] = self.ewma[i];
                something_new = true;
            }
        }
        something_new
    }
}

fn main() {
    let mut port = serialport::new("/dev/pactlers", 0) // such baudrate, much speed, wow
        .timeout(Duration::from_millis(10))
        .open()
        .expect("Failed to open port");

    let mut buf: [u8; 2 * N_ADCS] = [0; 2 * N_ADCS];
    let mut values = Tewmas::new(buf);
    let mut old_values = [0; N_ADCS];
    loop {
        let count = port.read(&mut buf).expect("Found no data!");
        if count == 10 {
            if values.update(buf) {
                for (i, &v) in values.tewma.iter().enumerate() {
                    if v != old_values[i] {
                        match i {
                            0 => println!("{} → {}", i, v),
                            1 => println!("{} → {}", i, v),
                            2 => println!("{} → {}", i, v),
                            3 => println!("{} → {}", i, v),
                            _ => println!("{} → {}", i, v),
                        }
                    }
                }
                old_values = values.tewma;
            }
        } else {
            println!("wrong read count: {}", count);
        }
    }
}
