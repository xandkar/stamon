use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, Context, Result};

#[cfg(test)]
mod tests;

#[derive(Copy, Clone, Debug, PartialEq)]
enum Direction {
    Increasing,
    Decreasing,
    Full,
    Unknown,
}

impl Direction {
    fn to_char(self) -> char {
        match self {
            Self::Increasing => '>',
            Self::Decreasing => '<',
            Self::Full => '=',
            Self::Unknown => '?',
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
enum BatteryState {
    PendingCharge,
    Charging,
    Discharging,
    FullyCharged,
    Unexpected,
}

impl std::str::FromStr for BatteryState {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let s = match s {
            "pending-charge" => Self::PendingCharge,
            "charging" => Self::Charging,
            "fully-charged" => Self::FullyCharged,
            "discharging" => Self::Discharging,
            s => {
                tracing::warn!("unexpected battery state: {:?}", s);
                Self::Unexpected
            }
        };
        Ok(s)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Battery {
    path: String, // TODO Try &str
    state: BatteryState,
    energy: f32,
    energy_full: f32,
}

#[derive(Debug, Clone, PartialEq)]
struct LinePower {
    path: String, // TODO Try &str
    online: bool,
}

#[derive(Debug)]
enum MsgIntermediate {
    Device {
        path: String,                // TODO Try &str
        native_path: Option<String>, // TODO Try &str
    },
    LinePower(LinePower),
    Battery {
        path: String, // TODO Try &str
        state: Option<BatteryState>,
        energy: Option<f32>,
        energy_full: Option<f32>,
    },
    Unhandled,
}

#[derive(Debug, PartialEq)]
enum Msg {
    LinePower(LinePower),
    Battery(Battery),
}

struct Messages<'a> {
    lines: Box<dyn Iterator<Item = String> + 'a>, // TODO Try &str
}

impl<'a> Messages<'a> {
    fn from_output_lines(
        lines: Box<dyn Iterator<Item = String> + 'a>,
    ) -> Self {
        Self { lines }
    }

    fn parse_next(&mut self) -> Result<Option<Msg>> {
        let mut msg: Option<MsgIntermediate> = None;
        loop {
            match self.lines.next() {
                None => return Ok(None),
                Some(line) => {
                    let fields =
                        line.split_whitespace().collect::<Vec<&str>>();
                    tracing::trace!("Line fields: {:?}", &fields);
                    match (line.starts_with([' ', '\t']), &msg, &fields[..]) {
                        // end msg
                        (false, _, [] | ["Monitoring", "activity", ..]) => {
                            match &msg {
                                Some(MsgIntermediate::LinePower(lp)) => {
                                    return Ok(Some(Msg::LinePower(
                                        lp.clone(),
                                    )))
                                }
                                Some(
                                    imsg @ MsgIntermediate::Battery {
                                        path,
                                        state,
                                        energy,
                                        energy_full,
                                    },
                                ) => {
                                    let state = state.ok_or_else(|| {
                                        anyhow!("missing state: {:?}", imsg)
                                    })?;
                                    let energy = energy.ok_or_else(|| {
                                        anyhow!("missing energy: {:?}", imsg)
                                    })?;
                                    let energy_full = energy_full
                                        .ok_or_else(|| {
                                            anyhow!(
                                                "missing energy_full: {:?}",
                                                imsg
                                            )
                                        })?;
                                    if energy > energy_full {
                                        return Err(anyhow!(
                                            "energy exceeds energy_full ({} > {}) for battery path: {:?}",
                                            energy, energy_full, &path
                                        ));
                                    }
                                    if energy < 0.0 {
                                        return Err(anyhow!(
                                            "negative energy ({}) for battery path: {:?}",
                                            energy, &path
                                        ));
                                    };
                                    if energy_full < 0.0 {
                                        return Err(anyhow!(
                                            "negative energy_full ({}) for battery path: {:?}",
                                            energy_full, &path
                                        ));
                                    };
                                    let msg = Msg::Battery(Battery {
                                        path: path.clone(),
                                        state,
                                        energy,
                                        energy_full,
                                    });
                                    return Ok(Some(msg));
                                }
                                Some(_) => msg = None,
                                None => (),
                            }
                        }

                        // new msg - device
                        (false, None, ["Device:", path]) => {
                            msg = Some(MsgIntermediate::Device {
                                path: path.to_string(),
                                native_path: None,
                            });
                        }

                        (
                            false,
                            None,
                            [_timestamp, "device", "changed:", path],
                        ) => {
                            msg = Some(MsgIntermediate::Device {
                                path: path.to_string(),
                                native_path: None,
                            });
                        }

                        // new msg - unhandled
                        (false, None, _) => {
                            msg = Some(MsgIntermediate::Unhandled);
                        }

                        // msg fields
                        (
                            true,
                            Some(MsgIntermediate::Device {
                                path,
                                native_path: None,
                            }),
                            ["native-path:", native_path],
                        ) => {
                            msg = Some(MsgIntermediate::Device {
                                path: path.clone(),
                                native_path: Some(native_path.to_string()),
                            });
                        }

                        // -- BEGIN battery
                        (
                            true,
                            Some(MsgIntermediate::Device {
                                path,
                                native_path,
                            }),
                            ["battery"],
                        ) => {
                            msg = Some(MsgIntermediate::Battery {
                                path: match native_path {
                                    None => path.to_string(),
                                    Some(path) => path.to_string(),
                                },
                                state: None,
                                energy: None,
                                energy_full: None,
                            });
                        }
                        (
                            true,
                            Some(MsgIntermediate::Battery {
                                path: p,
                                state: _,
                                energy: e,
                                energy_full: ef,
                            }),
                            ["state:", state],
                        ) => {
                            msg = Some(MsgIntermediate::Battery {
                                path: p.clone(),
                                state: Some(
                                    state.parse::<BatteryState>().context(
                                        format!("line: {:?}", &line),
                                    )?,
                                ),
                                energy: *e,
                                energy_full: *ef,
                            });
                        }
                        (
                            true,
                            Some(MsgIntermediate::Battery {
                                path: p,
                                state: s,
                                energy: _,
                                energy_full: ef,
                            }),
                            ["energy:", qty, _units],
                        ) => {
                            msg = Some(MsgIntermediate::Battery {
                                path: p.clone(),
                                state: *s,
                                energy: Some(
                                    qty.parse::<f32>().context(format!(
                                        "line: {:?}",
                                        &line
                                    ))?,
                                ),
                                energy_full: *ef,
                            });
                        }
                        (
                            true,
                            Some(MsgIntermediate::Battery {
                                path: p,
                                state: s,
                                energy: e,
                                energy_full: _,
                            }),
                            ["energy-full:", qty, _units],
                        ) => {
                            msg = Some(MsgIntermediate::Battery {
                                path: p.clone(),
                                state: *s,
                                energy: *e,
                                energy_full: Some(
                                    qty.parse::<f32>().context(format!(
                                        "line: {:?}",
                                        &line
                                    ))?,
                                ),
                            });
                        }
                        // -- END battery

                        // -- BEGIN line-power
                        (
                            true,
                            Some(MsgIntermediate::Device {
                                path,
                                native_path,
                            }),
                            ["line-power"],
                        ) => {
                            msg =
                                Some(MsgIntermediate::LinePower(LinePower {
                                    path: match native_path {
                                        None => path.to_string(),
                                        Some(path) => path.to_string(),
                                    },
                                    online: false,
                                }));
                        }
                        (
                            true,
                            Some(MsgIntermediate::LinePower(LinePower {
                                path,
                                ..
                            })),
                            ["online:", online],
                        ) => {
                            msg =
                                Some(MsgIntermediate::LinePower(LinePower {
                                    path: path.clone(),
                                    online: match *online {
                                        "yes" => true,
                                        "no" => false,
                                        _ => {
                                            return Err(anyhow!(
                                                "Unexpected value for \"online\": {:?}",
                                                online
                                            )
                                            .context(format!(
                                                "line: {:?}",
                                                &line
                                            )))
                                        }
                                    },
                                }));
                        }
                        // -- END line-power

                        // unused
                        (true, Some(_), _) => {
                            tracing::trace!(
                                "Ignoring msg with fields: {:?}",
                                &fields
                            );
                        }

                        // unexpected
                        _ => {
                            tracing::warn!("Unhandled line: {:?}", &line);
                        }
                    }
                }
            }
        }
    }
}

impl<'a> Iterator for Messages<'a> {
    type Item = Msg;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.parse_next() {
                Ok(None) => return None,
                Ok(Some(msg)) => return Some(msg),
                Err(e) => {
                    tracing::error!("Failure to parse a message: {:?}", e);
                }
            }
        }
    }
}

mod alert {
    use notify_rust::{Notification, Urgency};

    pub enum Level {
        Lo,
        Mid,
        Hi,
    }

    pub struct Alert {
        notification: Notification,
    }

    impl Alert {
        pub fn new(level: Level, threshold: u64, current: u64) -> Self {
            let mut notification = Notification::new();
            notification
                .summary(&format!("Battery power bellow {}%!", threshold))
                .body(&format!("{}%", current))
                .urgency(match level {
                    Level::Lo => Urgency::Low,
                    Level::Mid => Urgency::Normal,
                    Level::Hi => Urgency::Critical,
                });
            Self { notification }
        }
    }

    impl crate::Alert for Alert {
        fn send(&self) -> anyhow::Result<()> {
            self.notification.show()?;
            Ok(())
        }
    }
}

#[derive(Debug)]
struct State {
    // TODO Alerts.
    prefix: String,
    plugged_in: bool,
    batteries: HashMap<String, Battery>, // TODO Try &str
    alerts_init: Vec<u64>,
    alerts_curr: Vec<u64>,
}

impl State {
    fn new(prefix: &str, alert_triggers: &[u64]) -> Result<Self> {
        match alert_triggers.iter().find(|n| **n > 100) {
            Some(n) => {
                Err(anyhow!("Alert value out of percentage range: {:?}", n))
            }
            None => Ok(Self {
                prefix: prefix.to_owned(),
                plugged_in: false,
                batteries: HashMap::new(),
                alerts_init: alert_triggers.to_vec(),
                alerts_curr: alert_triggers.to_vec(),
            }),
        }
    }

    fn alerts(&mut self) -> Option<Vec<Box<dyn crate::Alert>>> {
        match (self.direction(), self.percentage()) {
            (Direction::Decreasing, Some(pct)) => {
                let (mut triggered, remaining): (Vec<u64>, Vec<u64>) = self
                    .alerts_curr
                    .iter()
                    .partition(|threshold| threshold > &&pct);
                self.alerts_curr = remaining;
                triggered.sort();
                if let Some(threshold) = triggered.first() {
                    let level = match () {
                        // TODO User-specifyable alert urgency levels.
                        _ if *threshold <= 25 => alert::Level::Hi,
                        _ if *threshold <= 50 => alert::Level::Mid,
                        _ if *threshold <= 100 => alert::Level::Lo,
                        _ => unreachable!(
                            "Threshold value out of range: {:?}",
                            threshold
                        ),
                    };
                    Some(vec![Box::new(alert::Alert::new(
                        level, *threshold, pct,
                    ))])
                } else {
                    None
                }
            }
            _ => {
                // TODO Reset elsewhere, to optimize common case.
                self.alerts_curr = self.alerts_init.clone();
                None
            }
        }
    }

    fn direction(&self) -> Direction {
        if !self.plugged_in {
            tracing::debug!("Direction::Decreasing because not plugged-in.");
            Direction::Decreasing
        } else {
            tracing::debug!("Batteries: {:?}", self.batteries);
            let states: HashSet<BatteryState> =
                HashSet::from_iter(self.batteries.values().map(|b| b.state));
            if states.is_empty() {
                tracing::warn!(
                    "Direction::Unknown because plugged-in, but battery states are empty: {:?}",
                    states
                );
                Direction::Unknown
            } else if states.contains(&BatteryState::Discharging) {
                // TODO Should this be some sort of an alert?
                tracing::warn!(
                    "Direction::Decreasing because plugged-in, but battery states contain Discharging: {:?}",
                    states
                );
                Direction::Decreasing
            } else if states.contains(&BatteryState::PendingCharge) {
                tracing::debug!(
                    "Direction::Decreasing because plugged-in, but battery states contain PendingCharge: {:?}",
                    states
                );
                Direction::Decreasing
            } else if states.contains(&BatteryState::Charging) {
                tracing::debug!(
                    "Direction::Increasing because plugged-in and battery states contain Charging: {:?}",
                    states
                );
                Direction::Increasing
            } else if 0
                == states
                    .difference(&HashSet::from([BatteryState::FullyCharged]))
                    .count()
            {
                tracing::debug!(
                    "Direction::Full because plugged-in and battery states contain only FullyCharged: {:?}",
                    states
                );
                Direction::Full
            } else {
                tracing::warn!(
                    "Direction::Unknown because battery states are in a strange combination: {:?}",
                    states
                );
                Direction::Unknown
            }
        }
    }

    fn percentage(&self) -> Option<u64> {
        (!self.batteries.is_empty()).then_some(()).and_then(|()| {
            let cur = self.batteries.values().map(|b| b.energy).sum();
            let tot = self.batteries.values().map(|b| b.energy_full).sum();
            crate::math::percentage_floor(cur, tot)
        })
    }
}

impl crate::State for State {
    type Event = Msg;

    fn update(
        &mut self,
        msg: Self::Event,
    ) -> Result<Option<Vec<Box<dyn crate::Alert>>>> {
        match msg {
            Msg::Battery(b) if b.path.ends_with("/DisplayDevice") => {
                tracing::warn!(
                    "Ignoring the aggregate from 'upower --dump': {:?}",
                    b
                );
            }
            Msg::Battery(b) => {
                self.batteries.insert(b.path.clone(), b);
            }
            Msg::LinePower(LinePower { online, .. }) => {
                self.plugged_in = online;
            }
        }
        Ok(self.alerts())
    }

    fn display<W: std::io::Write>(&self, mut buf: W) -> Result<()> {
        write!(buf, "{}{}", &self.prefix, self.direction().to_char())?;
        match self.percentage() {
            None => write!(buf, "---%")?,
            Some(pct) => write!(buf, "{:3.0}%", pct)?,
        }
        writeln!(buf)?;
        Ok(())
    }
}

fn run_monitor() -> Result<impl Iterator<Item = String>> {
    let dump = crate::process::spawn("upower", &["--dump"])?;
    let monitor = crate::process::spawn("upower", &["--monitor-detail"])?;
    let lines = dump.chain(monitor).map_while(|line_result| {
        line_result
            .map_err(|e| {
                tracing::error!("Failed to read upower output: {:?}", e);
                e
            })
            .ok()
    });
    Ok(lines)
}

fn messages<'a>() -> Result<impl Iterator<Item = Msg> + 'a> {
    // TODO Revisit if we really need to box lines.
    let lines = run_monitor()?;
    let messages = Messages::from_output_lines(Box::new(lines));
    Ok(messages)
}

pub fn run(prefix: &str, alert_triggers: &[u64]) -> Result<()> {
    crate::pipeline_to_stdout(
        messages()?,
        State::new(prefix, alert_triggers)?,
    )
}
