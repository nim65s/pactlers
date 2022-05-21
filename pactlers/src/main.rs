use std::{process::Command, time::Duration};

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

enum PactlClass {
    App,
    Mic,
    Spk,
}

impl PactlClass {
    fn class(&self) -> &str {
        match &self {
            PactlClass::App => "sink-input",
            PactlClass::Mic => "source",
            PactlClass::Spk => "sink",
        }
    }
    fn id(&self) -> &str {
        match &self {
            PactlClass::App => "Sink Input #",
            PactlClass::Mic => "Source #",
            PactlClass::Spk => "Sink #",
        }
    }
    fn name(&self) -> &str {
        match &self {
            PactlClass::App => "application.name = ",
            _ => "alsa.card_name = ",
        }
    }
}

struct PactlChannel<'a> {
    class: PactlClass,
    name: &'a str,
}

impl<'a> PactlChannel<'a> {
    fn new(class: PactlClass, name: &'a str) -> Self {
        Self { class, name }
    }

    fn set(&self, vol: u32) {
        // Adc: 0-4095 ; pactl: 0-65536 -> x16
        let vol = &((vol + 1) * 16).to_string();
        let output = Command::new("pactl")
            .args(["list", &format!("{}s", self.class.class())])
            .output()
            .expect("Failed to execute command");
        let output = std::str::from_utf8(&output.stdout).expect("invalid utf-8");
        let mut id = "";
        let mut childs = vec![];
        for line in output.split('\n') {
            let line = line.trim();
            if let Some(line) = line.strip_prefix(self.class.id()) {
                id = line;
            } else if let Some(line) = line.strip_prefix(self.class.name()) {
                if line.contains(self.name) {
                    childs.push(
                        Command::new("pactl")
                            .args([&format!("set-{}-volume", self.class.class()), id, vol])
                            .spawn()
                            .expect("Failed to start command"),
                    );
                }
            }
        }
        for mut child in childs {
            child.wait().expect("Failed to run command");
        }
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

    let chans: [PactlChannel; N_ADCS] = [
        PactlChannel::new(PactlClass::App, "snapclient"),
        PactlChannel::new(PactlClass::App, "Firefox"),
        PactlChannel::new(PactlClass::App, "VLC"),
        PactlChannel::new(PactlClass::Mic, "BIRD"),
        PactlChannel::new(PactlClass::Spk, "HDA"),
    ];

    loop {
        let count = port.read(&mut buf).expect("Found no data!");
        if count == 10 {
            if values.update(buf) {
                for (i, &v) in values.tewma.iter().enumerate() {
                    if v != old_values[i] {
                        chans[i].set(v as u32);
                    }
                }
                old_values = values.tewma;
            }
        } else {
            println!("wrong read count: {}", count);
        }
    }
}
