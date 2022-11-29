use std::process::Command;

pub enum PactlClass {
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

pub struct PactlChannel<'a> {
    class: PactlClass,
    name: &'a str,
}

impl<'a> PactlChannel<'a> {
    pub fn new(class: PactlClass, name: &'a str) -> Self {
        Self { class, name }
    }

    pub fn set(&self, vol: u16) {
        // Adc: 0-4095 ; pactl: 0-65536 -> x16
        let vol = &((vol as u32 + 1) * 16).to_string();
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
