mod bluetoothctl;

use std::{fs, time::Duration};

use anyhow::{anyhow, Result};

struct State<'a> {
    prefix: &'a str,
    postfix: &'a str,
    device_state: Option<ControllerState>,
    next_pos_of_device_to_display: usize,
}

impl<'a> State<'a> {
    fn new(prefix: &'a str, postfix: &'a str) -> Self {
        Self {
            prefix,
            postfix,
            device_state: None,
            next_pos_of_device_to_display: 0,
        }
    }
}

impl<'a> crate::pipeline::State for State<'a> {
    type Event = Option<ControllerState>;

    fn update(
        &mut self,
        dev_opt: Self::Event,
    ) -> Result<Option<Vec<crate::alert::Alert>>> {
        self.device_state = dev_opt;
        Ok(None)
    }

    fn display<W: std::io::Write>(&mut self, mut buf: W) -> Result<()> {
        write!(buf, "{}", self.prefix)?;
        match self.device_state {
            Some(ControllerState::NoDev) | None => {
                self.next_pos_of_device_to_display = 0;
                write!(buf, " ")?
            }
            // TODO Distinguish between OffSoft and OffHard
            Some(ControllerState::OffSoft | ControllerState::OffHard) => {
                self.next_pos_of_device_to_display = 0;
                write!(buf, "-")?
            }
            Some(ControllerState::On { devices: None }) => {
                self.next_pos_of_device_to_display = 0;
                write!(buf, "+")?
            }
            Some(ControllerState::On {
                devices: Some(ref devices),
            }) => {
                let bat_pcts: Vec<u8> = devices
                    .into_iter()
                    .filter_map(|dev| dev.bat_pct)
                    .collect();
                let n = bat_pcts.len();
                let i = self.next_pos_of_device_to_display;
                self.next_pos_of_device_to_display = i.wrapping_add(1);
                match (n > 0).then(|| bat_pcts.get(i % n)).flatten() {
                    None => {
                        write!(buf, "{n}")?;
                    }
                    Some(bat_pct) => {
                        write!(buf, "{n} {bat_pct:3.0}%",)?;
                    }
                }
            }
        };
        writeln!(buf, "{}", self.postfix)?;
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum Details {
    No,
    Yes { timeout: Duration },
}

#[derive(Debug)]
struct Device {
    _id: String,
    bat_pct: Option<u8>,
}

#[derive(Debug)]
enum ControllerState {
    NoDev,
    OffHard,
    OffSoft,
    On { devices: Option<Vec<Device>> },
}

impl ControllerState {
    fn read(details: Details) -> Result<Option<Self>> {
        // This method of device state lookup is taken from TLP bluetooth command.
        // TODO Checkout https://crates.io/crates/bluer
        let mut bt_state_opt: Option<Self> = None;
        for entry in fs::read_dir("/sys/class/rfkill/")? {
            let entry = entry?;
            let mut path_type = entry.path();
            let mut path_state = entry.path();
            path_type.push("type");
            path_state.push("state");
            if let "bluetooth" = fs::read_to_string(path_type)?.trim_end() {
                let state_byte: u8 =
                    fs::read_to_string(path_state)?.trim_end().parse()?;
                bt_state_opt = Some(Self::from_byte(state_byte, details)?);
                return Ok(bt_state_opt);
            }
        }
        Ok(bt_state_opt)
    }

    fn from_byte(b: u8, details: Details) -> Result<Self> {
        let selph = match b {
            0 => Self::OffSoft,
            1 => {
                let devices = match details {
                    Details::No => None,
                    Details::Yes { timeout } => {
                        let devices = fetch_devices(timeout);
                        Some(devices)
                    }
                };
                Self::On { devices }
            }
            2 => Self::OffHard,
            254 => Self::NoDev,
            _ => return Err(anyhow!("Invalid state byte: {:?}", b)),
        };
        Ok(selph)
    }
}

fn fetch_devices(timeout: Duration) -> Vec<Device> {
    let dev_ids = bluetoothctl::devices_connected(timeout)
        .ok()
        .unwrap_or_default();
    dev_ids
        .into_iter()
        .map(|id| {
            let info = bluetoothctl::info(&id, timeout).ok();
            let bat_pct = info.map(|i| i.bat_pct).flatten();
            Device { _id: id, bat_pct }
        })
        .collect()
}

pub fn run(
    prefix: &str,
    postfix: &str,
    interval: Duration,
    details_enabled: bool,
    timeout: Duration,
) -> Result<()> {
    use crate::clock;

    let details = if details_enabled {
        Details::Yes { timeout }
    } else {
        Details::No
    };

    let events = clock::new(interval)
        .map(|clock::Tick| ControllerState::read(details))
        .filter_map(|result| match result {
            Err(error) => {
                tracing::error!(?error, "Failed to read device state.");
                None
            }
            Ok(dev_opt) => Some(dev_opt),
        });
    crate::pipeline::run_to_stdout(events, State::new(prefix, postfix))
}
