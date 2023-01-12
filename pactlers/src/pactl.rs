use std::process::Command;

use crate::error::Error;

pub enum Class {
    App,
    Mic,
    Spk,
}

impl Class {
    const fn class(&self) -> &str {
        match &self {
            Self::App => "sink-input",
            Self::Mic => "source",
            Self::Spk => "sink",
        }
    }
    const fn id(&self) -> &str {
        match &self {
            Self::App => "Sink Input #",
            Self::Mic => "Source #",
            Self::Spk => "Sink #",
        }
    }
    const fn name(&self) -> &str {
        match &self {
            Self::App => "application.name = ",
            _ => "alsa.card_name = ",
        }
    }
}

pub struct Channel<'a> {
    class: Class,
    name: &'a str,
}

impl<'a> Channel<'a> {
    pub const fn new(class: Class, name: &'a str) -> Self {
        Self { class, name }
    }

    pub fn set(&self, vol: u16) -> Result<(), Error> {
        // Adc: 0-4095 ; pactl: 0-65536 -> x16
        let vol = &((u32::from(vol) + 1) * 16).to_string();
        let output = Command::new("pactl")
            .args(["list", &format!("{}s", self.class.class())])
            .output()?;
        let output = std::str::from_utf8(&output.stdout)?;
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
                            .spawn()?,
                    );
                }
            }
        }
        for mut child in childs {
            child.wait()?;
        }
        Ok(())
    }
}
