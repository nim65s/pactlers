use async_channel::unbounded;
use futures::stream::StreamExt;
use pactlers_lib::{Cmd, N_ADCS};
use std::path::Path;
use std::time::Duration;
use tokio::{task, time::sleep};
use tokio_serial::SerialPortBuilderExt;
use tokio_util::codec::Decoder;

mod pactl;
mod sercon;

use crate::pactl::*;
use crate::sercon::SerialConnection;

const DEV: &str = "/dev/pactlers";

#[tokio::main]
async fn main() {
    let (tx, rx) = unbounded::<Cmd>();

    let ctrl_task = task::spawn(async move {
        let chans: [PactlChannel; N_ADCS] = [
            PactlChannel::new(PactlClass::Spk, "HDA"),
            PactlChannel::new(PactlClass::Mic, "BIRD"),
            PactlChannel::new(PactlClass::App, "VLC"),
            PactlChannel::new(PactlClass::App, "Firefox"),
            PactlChannel::new(PactlClass::App, "snapclient"),
        ];
        loop {
            if let Ok(cmd) = rx.recv().await {
                chans[cmd.select as usize].set(cmd.volume);
            }
            sleep(Duration::from_millis(1)).await;
        }
    });

    println!("Opening {} ...", DEV);

    while Path::new(DEV).exists() && !ctrl_task.is_finished() {
        let port = tokio_serial::new(DEV, 0) // such baudrate, much speed, wow
            .open_native_async()
            .expect("Failed to open serial port.");
        println!("Connected.");
        let (_uart_writer, mut uart_reader) = SerialConnection::new().framed(port).split();

        while let Some(Ok(cmd)) = uart_reader.next().await {
            tx.send(cmd).await.unwrap();
        }
        eprintln!("Disconnected.");
        sleep(Duration::from_secs(1)).await;
    }

    eprintln!("{} not available.", DEV);
    ctrl_task.abort();
}
