use std::{sync::mpsc, thread, time::Duration};

mod pactl;

use crate::pactl::*;

const N_ADCS: usize = 5;

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
                //println!("{}: {}", buf[0], v);
                last = buf;
            }
            thread::sleep(Duration::from_millis(1));
        }
    });

    println!("Opening /dev/pactlers ...");

    loop {
        let port = serialport::new("/dev/pactlers", 0) // such baudrate, much speed, wow
            .timeout(Duration::from_secs(3600 * 24 * 7))
            .open();
        if port.is_err() {
            thread::sleep(Duration::from_millis(500));
            continue;
        }
        println!("Connected.");
        let mut port = port.unwrap();

        let mut buf: [u8; 3] = [0; 3];

        loop {
            match port.read(&mut buf) {
                Ok(3) => tx.send(buf).unwrap(),
                Ok(count) => eprintln!("wrong read count: {}", count),
                Err(_) => break,
            }
            thread::sleep(Duration::from_millis(1));
        }
        eprintln!("Disconnected.");
    }
}
