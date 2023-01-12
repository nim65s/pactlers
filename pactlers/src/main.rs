use async_channel::unbounded;
use futures::stream::StreamExt;
use pactlers_lib::{Cmd, N_ADCS};
use std::path::Path;
use std::time::Duration;
use tokio::{task, time::sleep};
use tokio_serial::SerialPortBuilderExt;
use tokio_util::codec::Decoder;

mod error;
mod pactl;
mod sercon;

use crate::error::Error;
use crate::pactl::{Channel, Class};
use crate::sercon::SerialConnection;

const DEV: &str = "/dev/pactlers";

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (tx, rx) = unbounded::<Cmd>();

    let ctrl_task = task::spawn(async move {
        let chans: [Channel; N_ADCS] = [
            Channel::new(Class::Spk, "HDA"),
            Channel::new(Class::Mic, "BIRD"),
            Channel::new(Class::App, "VLC"),
            Channel::new(Class::App, "Firefox"),
            Channel::new(Class::App, "snapclient"),
        ];
        loop {
            let cmd = rx.recv().await?;
            chans[cmd.select as usize].set(cmd.volume)?;
            sleep(Duration::from_millis(1)).await;
        }
        #[allow(unreachable_code)]
        Ok::<(), Error>(())
    });

    println!("Opening {DEV} ...");

    while Path::new(DEV).exists() && !ctrl_task.is_finished() {
        let port = tokio_serial::new(DEV, 0) // such baudrate, much speed, wow
            .open_native_async()?;
        println!("Connected.");
        let (_uart_writer, mut uart_reader) = SerialConnection::new().framed(port).split();

        while let Some(Ok(cmd)) = uart_reader.next().await {
            tx.send(cmd).await?;
        }
        eprintln!("Disconnected.");
        sleep(Duration::from_secs(1)).await;
    }

    eprintln!("{DEV} not available.");
    ctrl_task.abort();
    Ok(())
}
