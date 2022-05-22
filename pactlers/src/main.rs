use std::{process::Command, sync::mpsc, thread, time::Duration};

const N_ADCS: usize = 5;

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
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let chans: [PactlChannel; N_ADCS] = [
            PactlChannel::new(PactlClass::Spk, "HDA"),
            PactlChannel::new(PactlClass::Mic, "BIRD"),
            PactlChannel::new(PactlClass::App, "VLC"),
            PactlChannel::new(PactlClass::App, "Firefox"),
            PactlChannel::new(PactlClass::App, "snapclient"),
        ];
        let mut buf: [u8; 3] = [0; 3];
        let mut last: [u8; 3] = [0; 3];
        loop {
            while let Ok(b) = rx.try_recv() {
                buf = b;
            }
            if buf != last {
                let v = u32::from_le_bytes([buf[2], buf[1], 0, 0]);
                chans[buf[0] as usize].set(v);
                println!("{}: {}", buf[0], v);
                last = buf;
            }
        }
    });

    let mut port = serialport::new("/dev/pactlers", 0) // such baudrate, much speed, wow
        .timeout(Duration::from_secs(3600 * 24 * 7))
        .open()
        .expect("Failed to open port");

    let mut buf: [u8; 3] = [0; 3];

    loop {
        let count = port.read(&mut buf).expect("Found no data!");
        if count == 3 {
            tx.send(buf).unwrap();
        } else {
            println!("wrong read count: {}", count);
        }
    }
}
