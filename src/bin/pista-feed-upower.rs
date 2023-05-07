use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, Context, Result};

#[derive(clap::Parser, Debug)]
struct Cli {
    #[clap(long = "prefix", default_value = "⚡ ")]
    prefix: String,
}

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
    state: Option<BatteryState>,
    energy: Option<f32>,
    energy_full: Option<f32>,
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
    Battery(Battery),
    Unhandled,
}

#[derive(Debug, PartialEq)]
enum Msg {
    LinePower(LinePower),
    Battery(Battery),
}

struct Messages<'a> {
    lines: &'a mut dyn Iterator<Item = String>, // TODO Try &str
}

impl<'a> Messages<'a> {
    fn from_output_lines(lines: &'a mut dyn Iterator<Item = String>) -> Self {
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
                                Some(MsgIntermediate::Battery(bat)) => {
                                    return Ok(Some(Msg::Battery(
                                        bat.clone(),
                                    )))
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
                            })
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
                            msg = Some(MsgIntermediate::Battery(Battery {
                                path: match native_path {
                                    None => path.to_string(),
                                    Some(path) => path.to_string(),
                                },
                                state: None,
                                energy: None,
                                energy_full: None,
                            }))
                        }
                        (
                            true,
                            Some(MsgIntermediate::Battery(Battery {
                                path: p,
                                state: _,
                                energy: e,
                                energy_full: ef,
                            })),
                            ["state:", state],
                        ) => {
                            msg = Some(MsgIntermediate::Battery(Battery {
                                path: p.clone(),
                                state: Some(
                                    state.parse::<BatteryState>().context(
                                        format!("line: {:?}", &line),
                                    )?,
                                ),
                                energy: *e,
                                energy_full: *ef,
                            }))
                        }
                        (
                            true,
                            Some(MsgIntermediate::Battery(Battery {
                                path: p,
                                state: s,
                                energy: _,
                                energy_full: ef,
                            })),
                            ["energy:", qty, _units],
                        ) => {
                            msg = Some(MsgIntermediate::Battery(Battery {
                                path: p.clone(),
                                state: *s,
                                energy: Some(
                                    qty.parse::<f32>().context(format!(
                                        "line: {:?}",
                                        &line
                                    ))?,
                                ),
                                energy_full: *ef,
                            }))
                        }
                        (
                            true,
                            Some(MsgIntermediate::Battery(Battery {
                                path: p,
                                state: s,
                                energy: e,
                                energy_full: _,
                            })),
                            ["energy-full:", qty, _units],
                        ) => {
                            msg = Some(MsgIntermediate::Battery(Battery {
                                path: p.clone(),
                                state: *s,
                                energy: *e,
                                energy_full: Some(
                                    qty.parse::<f32>().context(format!(
                                        "line: {:?}",
                                        &line
                                    ))?,
                                ),
                            }))
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
                                }))
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
                                }))
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

impl Iterator for Messages<'_> {
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

type StateAggregate = (Direction, f32);

#[derive(Debug)]
struct State {
    plugged_in: bool,
    batteries: HashMap<String, Battery>, // TODO Try &str
}

impl State {
    fn new() -> Self {
        Self {
            plugged_in: false,
            batteries: HashMap::new(),
        }
    }

    fn update(&mut self, msg: Msg) {
        match msg {
            Msg::Battery(b) if b.path.ends_with("/DisplayDevice") => {
                tracing::warn!(
                    "Ignoring the aggregate from 'upower --dump': {:?}",
                    b
                )
            }
            Msg::Battery(b) => {
                self.batteries.insert(b.path.clone(), b);
            }
            Msg::LinePower(LinePower { online, .. }) => {
                self.plugged_in = online;
            }
        }
    }

    fn direction(&self) -> Direction {
        if !self.plugged_in {
            tracing::debug!("Direction::Decreasing because not plugged-in.");
            Direction::Decreasing
        } else {
            tracing::debug!("Batteries: {:?}", self.batteries);
            let states: HashSet<BatteryState> = HashSet::from_iter(
                self.batteries.values().filter_map(|b| b.state),
            );
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

    fn percentage(&self) -> f32 {
        let cur: f32 = self
            .batteries
            .values()
            .map(|b| b.energy.unwrap_or(0.0))
            .sum();
        let tot: f32 = self
            .batteries
            .values()
            .map(|b| b.energy_full.unwrap_or(0.0))
            .sum();
        (cur / tot) * 100.0
    }

    fn aggregate(&self) -> StateAggregate {
        (self.direction(), self.percentage())
    }
}

struct StateAggregates<'a> {
    state: State,
    messages: &'a mut dyn Iterator<Item = Msg>,
}

impl<'a> StateAggregates<'a> {
    fn from_messages(messages: &'a mut dyn Iterator<Item = Msg>) -> Self {
        Self {
            state: State::new(),
            messages,
        }
    }
}

impl<'a> Iterator for StateAggregates<'a> {
    type Item = StateAggregate;

    fn next(&mut self) -> Option<Self::Item> {
        match self.messages.next() {
            None => None,
            Some(m) => {
                self.state.update(m);
                Some(self.state.aggregate())
            }
        }
    }
}

pub fn upower_run() -> Result<impl Iterator<Item = String>> {
    // TODO dump doesn't have to be spawned, but can be ran to completion
    //      before launching monitor.
    let dump = spawn("upower", &["--dump"])?;
    let monitor = spawn("upower", &["--monitor-detail"])?;
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

fn spawn(
    cmd: &str,
    args: &[&str],
) -> Result<impl Iterator<Item = Result<String, std::io::Error>>> {
    let stdout = std::process::Command::new("stdbuf")
        .args(["-o", "L"])
        .arg(cmd)
        .args(args)
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| anyhow!("Failed to spawn {cmd:?}: {e:?}"))?
        .stdout
        .ok_or_else(|| anyhow!("Failed to get stdout of: {:?}", cmd))?;
    let lines = {
        use std::io::BufRead; // .lines()
        std::io::BufReader::new(stdout).lines()
    };
    Ok(lines)
}

macro_rules! state_aggregates {
    () => {
        StateAggregates::from_messages(&mut Messages::from_output_lines(
            &mut upower_run()?,
        ))
    };
}

fn main() -> Result<()> {
    pista_feeds::tracing_init()?;
    let cli = {
        use clap::Parser;
        Cli::parse()
    };
    tracing::info!("cli: {:?}", &cli);
    let mut stdout = std::io::stdout().lock();
    for (direction, percentage) in state_aggregates!() {
        tracing::debug!(
            "Current: direction={:?}, percentage={:?}",
            direction,
            percentage,
        );
        // TODO Notify on negative state changes.
        if let Err(e) = {
            use std::io::Write;
            writeln!(
                stdout,
                "{}{}{:3.0}%",
                &cli.prefix,
                direction.to_char(),
                percentage.floor() // Show the worst case.
            )
        } {
            tracing::error!("Failed to write to stdout: {:?}", e)
        }
    }
    Err(anyhow!("upower exited"))
}

#[cfg(test)]
mod tests {
    use std::iter::zip;

    use super::*;

    #[test]
    fn dump() {
        let output: String =
            std::fs::read_to_string("tests/upower-dump.txt").unwrap();
        let mut lines = output.lines().map(|l| l.to_string());
        let messages_produced: Vec<Msg> =
            Messages::from_output_lines(&mut lines).collect();
        let messages_expected: Vec<Msg> = vec![
            Msg::LinePower(LinePower {
                path: "AC".to_string(),
                online: false,
            }),
            Msg::Battery(Battery {
                path: "BAT0".to_string(),
                state: Some(BatteryState::Discharging),
                energy: Some(87.2898),
                energy_full: Some(89.148),
            }),
            Msg::Battery(Battery {
                path: "/org/freedesktop/UPower/devices/DisplayDevice"
                    .to_string(),
                state: Some(BatteryState::Discharging),
                energy: Some(87.2898),
                energy_full: Some(89.148),
            }),
        ];
        assert_eq!(&messages_expected, &messages_produced);
        let states_produced: Vec<StateAggregate> =
            StateAggregates::from_messages(
                &mut messages_produced.into_iter(),
            )
            .collect();
        let states_expected = vec![
            (Direction::Decreasing, std::f32::NAN),
            (Direction::Decreasing, 97.9156),
            (Direction::Decreasing, 97.9156),
        ];

        // State aggregates cannot be compared directly, because they contain
        // floats and we do expect them to at least initially be NaN.
        for ((dir_expected, pct_expected), (dir_produced, pct_produced)) in
            zip(states_expected, states_produced)
        {
            assert_eq!(dir_expected, dir_produced);
            assert!(matches!(
                pct_expected.partial_cmp(&pct_produced),
                None | Some(std::cmp::Ordering::Equal),
            ));
        }
    }

    #[test]
    fn monitor() {
        let output: String =
            std::fs::read_to_string("tests/upower-monitor-detail.txt")
                .unwrap();
        let mut lines = output.lines().map(|l| l.to_string());
        let messages_produced: Vec<Msg> =
            Messages::from_output_lines(&mut lines).collect();
        dbg!(&messages_produced);
        let messages_expected: Vec<Msg> = vec![
            Msg::Battery(Battery {
                path: "BAT0".to_string(),
                state: Some(BatteryState::Discharging),
                energy: Some(42.8868),
                energy_full: Some(89.148),
            }),
            Msg::Battery(Battery {
                path: "BAT0".to_string(),
                state: Some(BatteryState::Discharging),
                energy: Some(42.8868),
                energy_full: Some(89.148),
            }),
            Msg::Battery(Battery {
                path: "BAT0".to_string(),
                state: Some(BatteryState::Discharging),
                energy: Some(42.8868),
                energy_full: Some(89.148),
            }),
            Msg::Battery(Battery {
                path: "BAT0".to_string(),
                state: Some(BatteryState::Discharging),
                energy: Some(42.8868),
                energy_full: Some(89.148),
            }),
            Msg::LinePower(LinePower {
                path: "AC".to_string(),
                online: false,
            }),
            Msg::LinePower(LinePower {
                path: "AC".to_string(),
                online: false,
            }),
        ];
        assert_eq!(&messages_expected, &messages_produced);
    }
}
