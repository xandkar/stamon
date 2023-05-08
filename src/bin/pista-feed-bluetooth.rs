// TODO Checkout https://crates.io/crates/bluer
use std::fs;

use anyhow::{anyhow, Result};
use clap::Parser;

#[derive(Debug, clap::Parser)]
struct Cli {
    #[clap(long = "interval", short = 'i', default_value = "2.0")]
    interval: f64,

    #[clap(long = "prefix", default_value = "ᛒ ")]
    prefix: String,
}

#[derive(Debug)]
enum State {
    OffSoft,
    On,
    OffHard,
    NoDev,
}

impl State {
    fn from_byte(b: u8) -> Result<Self> {
        match b {
            0 => Ok(Self::OffSoft),
            1 => Ok(Self::On),
            2 => Ok(Self::OffHard),
            254 => Ok(Self::NoDev),
            _ => Err(anyhow!("Invalid state byte: {:?}", b)),
        }
    }
}

fn bt_state() -> Result<Option<State>> {
    // This method of device state lookup is taken from TLP bluetooth command.
    let mut bt_state: Option<State> = None;
    for entry in fs::read_dir("/sys/class/rfkill/")? {
        let entry = entry?;
        let mut path_type = entry.path();
        let mut path_state = entry.path();
        path_type.push("type");
        path_state.push("state");
        if let "bluetooth" = fs::read_to_string(path_type)?.trim_end() {
            let state_byte: u8 =
                fs::read_to_string(path_state)?.trim_end().parse()?;
            bt_state = Some(State::from_byte(state_byte)?);
            return Ok(bt_state);
        }
    }
    Ok(bt_state)
}

// TODO Lookup number of connected devices.

fn main() -> Result<()> {
    pista_feeds::tracing_init()?;
    let cli = Cli::parse();
    tracing::info!("cli: {:?}", &cli);
    let interval = std::time::Duration::from_secs_f64(cli.interval);
    let mut stdout = std::io::stdout().lock();
    loop {
        match bt_state() {
            Err(e) => {
                tracing::error!(
                    "Failed to get bluetooth device state: {:?}",
                    e
                );
            }
            Ok(None) => {
                tracing::warn!("Did not find a bluetooth device");
            }
            Ok(Some(s)) => {
                if let Err(e) = {
                    use std::io::Write;
                    match s {
                        State::On => writeln!(stdout, "{}on ", &cli.prefix),
                        State::OffSoft => {
                            writeln!(stdout, "{}off", &cli.prefix)
                        }
                        State::OffHard => {
                            writeln!(stdout, "{}off", &cli.prefix)
                        }
                        State::NoDev => writeln!(stdout, "{}--", &cli.prefix),
                    }
                } {
                    tracing::error!("Failed to write to stdout: {:?}", e)
                }
            }
        }
        std::thread::sleep(interval);
    }
}
