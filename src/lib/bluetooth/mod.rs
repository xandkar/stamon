use std::fs;

use anyhow::{anyhow, Result};

#[derive(Debug)]
pub enum DeviceState {
    NoDev,
    OffHard,
    OffSoft,
    On,
}

impl DeviceState {
    // TODO Lookup number of connected devices.

    pub fn read() -> Result<Option<Self>> {
        // This method of device state lookup is taken from TLP bluetooth command.
        // TODO Checkout https://crates.io/crates/bluer
        let mut bt_state: Option<Self> = None;
        for entry in fs::read_dir("/sys/class/rfkill/")? {
            let entry = entry?;
            let mut path_type = entry.path();
            let mut path_state = entry.path();
            path_type.push("type");
            path_state.push("state");
            if let "bluetooth" = fs::read_to_string(path_type)?.trim_end() {
                let state_byte: u8 =
                    fs::read_to_string(path_state)?.trim_end().parse()?;
                bt_state = Some(Self::from_byte(state_byte)?);
                return Ok(bt_state);
            }
        }
        Ok(bt_state)
    }

    pub fn write<W: std::io::Write>(
        &self,
        mut buf: W,
        prefix: &str,
    ) -> Result<(), std::io::Error> {
        match self {
            Self::On => {
                writeln!(buf, "{}on ", prefix)
            }
            Self::OffSoft => {
                // TODO Distinguish from OffHard
                writeln!(buf, "{}off", prefix)
            }
            Self::OffHard => {
                // TODO Distinguish from OffSoft
                writeln!(buf, "{}off", prefix)
            }
            Self::NoDev => {
                writeln!(buf, "{}--", prefix)
            }
        }
    }

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
