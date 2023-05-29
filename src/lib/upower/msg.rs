use anyhow::{anyhow, Context, Result};

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum BatteryState {
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
pub struct Battery {
    pub path: String, // TODO Try &str
    pub state: BatteryState,
    pub energy: f32,
    pub energy_full: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LinePower {
    pub path: String, // TODO Try &str
    pub online: bool,
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
pub enum Msg {
    LinePower(LinePower),
    Battery(Battery),
}

impl Msg {
    fn from_lines(
        mut lines: impl Iterator<Item = String>,
    ) -> Result<Option<Self>> {
        let mut msg: Option<MsgIntermediate> = None;
        loop {
            match lines.next() {
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

pub struct Messages<'a> {
    lines: Box<dyn Iterator<Item = String> + 'a>, // TODO Try &str
}

impl<'a> Messages<'a> {
    pub fn from_lines(lines: impl Iterator<Item = String> + 'a) -> Self {
        Self {
            lines: Box::new(lines),
        }
    }

    pub fn from_run() -> Result<Self> {
        let output_lines = run_upower_monitor()?;
        Ok(Self::from_lines(output_lines))
    }
}

impl<'a> Iterator for Messages<'a> {
    type Item = Msg;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match Msg::from_lines(&mut self.lines) {
                Ok(None) => return None,
                Ok(Some(msg)) => return Some(msg),
                Err(e) => {
                    tracing::error!("Failure to parse a message: {:?}", e);
                }
            }
        }
    }
}

fn run_upower_monitor() -> Result<impl Iterator<Item = String>> {
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
