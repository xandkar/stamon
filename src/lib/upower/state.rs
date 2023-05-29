use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, Result};

use super::{alert, msg};

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

#[derive(Debug)]
pub struct State {
    // TODO Alerts.
    prefix: String,
    plugged_in: bool,
    batteries: HashMap<String, msg::Battery>, // TODO Try &str
    alerts_init: Vec<u64>,
    alerts_curr: Vec<u64>,
}

impl State {
    pub fn new(prefix: &str, alert_triggers: &[u64]) -> Result<Self> {
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
            let states: HashSet<msg::BatteryState> =
                HashSet::from_iter(self.batteries.values().map(|b| b.state));
            if states.is_empty() {
                tracing::warn!(
                    "Direction::Unknown because plugged-in, but \
                    battery states are empty: {:?}",
                    states
                );
                Direction::Unknown
            } else if states.contains(&msg::BatteryState::Discharging) {
                // TODO Should this be some sort of an alert?
                tracing::warn!(
                    "Direction::Decreasing because plugged-in, but \
                    battery states contain Discharging: {:?}",
                    states
                );
                Direction::Decreasing
            } else if states.contains(&msg::BatteryState::PendingCharge) {
                tracing::debug!(
                    "Direction::Decreasing because plugged-in, but \
                    battery states contain PendingCharge: {:?}",
                    states
                );
                Direction::Decreasing
            } else if states.contains(&msg::BatteryState::Charging) {
                tracing::debug!(
                    "Direction::Increasing because plugged-in and \
                    battery states contain Charging: {:?}",
                    states
                );
                Direction::Increasing
            } else if 0
                == states
                    .difference(&HashSet::from([
                        msg::BatteryState::FullyCharged,
                    ]))
                    .count()
            {
                tracing::debug!(
                    "Direction::Full because plugged-in and \
                    battery states contain only FullyCharged: {:?}",
                    states
                );
                Direction::Full
            } else {
                tracing::warn!(
                    "Direction::Unknown because battery states are \
                    in a strange combination: {:?}",
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
    type Event = msg::Msg;

    fn update(
        &mut self,
        msg: Self::Event,
    ) -> Result<Option<Vec<Box<dyn crate::Alert>>>> {
        match msg {
            msg::Msg::Battery(b) if b.path.ends_with("/DisplayDevice") => {
                tracing::warn!(
                    "Ignoring the aggregate from 'upower --dump': {:?}",
                    b
                );
            }
            msg::Msg::Battery(b) => {
                self.batteries.insert(b.path.clone(), b);
            }
            msg::Msg::LinePower(msg::LinePower { online, .. }) => {
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
