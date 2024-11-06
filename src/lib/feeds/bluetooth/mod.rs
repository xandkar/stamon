mod bluetoothctl;

use std::{fs, time::Duration};

use anyhow::{anyhow, Result};
use bluetoothctl::Info;

struct State<'a> {
    prefix: &'a str,
    device_state: Option<ControllerState>,
    next_pos_of_device_to_display: usize,
}

impl<'a> State<'a> {
    fn new(prefix: &'a str) -> Self {
        Self {
            prefix,
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
        match self.device_state {
            Some(ControllerState::NoDev) | None => {
                self.next_pos_of_device_to_display = 0;
                writeln!(buf, "{} ", self.prefix)?
            }
            // TODO Distinguish between OffSoft and OffHard
            Some(ControllerState::OffSoft | ControllerState::OffHard) => {
                self.next_pos_of_device_to_display = 0;
                writeln!(buf, "{}-", self.prefix)?
            }
            Some(ControllerState::On { details: None }) => {
                self.next_pos_of_device_to_display = 0;
                writeln!(buf, "{}+", self.prefix)?
            }
            Some(ControllerState::On {
                details: Some(ref details),
            }) => {
                let n = details.len();
                let i = self.next_pos_of_device_to_display;
                self.next_pos_of_device_to_display = i.wrapping_add(1);
                match (n > 0).then(|| details.get(i % n)).flatten() {
                    None
                    | Some((_, None))
                    | Some((
                        _,
                        Some(Info {
                            id: _,
                            bat_pct: None,
                        }),
                    )) => {
                        writeln!(buf, "{}{}", self.prefix, n)?;
                    }
                    Some((
                        _id,
                        Some(Info {
                            id: _,
                            bat_pct: Some(bat_pct),
                        }),
                    )) => {
                        writeln!(buf, "{}{} {}%", self.prefix, n, bat_pct)?;
                    }
                }
            }
        };
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum Details {
    No,
    Yes { timeout: Duration },
}

#[derive(Debug)]
enum ControllerState {
    NoDev,
    OffHard,
    OffSoft,
    On {
        details: Option<Vec<(String, Option<Info>)>>,
    },
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
                let details = match details {
                    Details::No => None,
                    Details::Yes { timeout } => Self::fetch_details(timeout),
                };
                Self::On { details }
            }
            2 => Self::OffHard,
            254 => Self::NoDev,
            _ => return Err(anyhow!("Invalid state byte: {:?}", b)),
        };
        Ok(selph)
    }

    fn fetch_details(
        timeout: Duration,
    ) -> Option<Vec<(String, Option<Info>)>> {
        bluetoothctl::devices_connected(timeout)
            .ok()
            .map(|dev_ids| {
                dev_ids
                    .into_iter()
                    .map(|id| {
                        let info = bluetoothctl::info(&id, timeout).ok();
                        (id, info)
                    })
                    .collect::<Vec<(String, Option<Info>)>>()
            })
    }
}

pub fn run(
    prefix: &str,
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
    crate::pipeline::run_to_stdout(events, State::new(prefix))
}
