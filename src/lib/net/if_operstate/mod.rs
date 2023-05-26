use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::Result;

#[derive(Debug)]
pub enum Status {
    Up,
    Down,
}

impl Status {
    pub fn read(operstate_path: &Path) -> Result<Option<Self>> {
        if operstate_path.exists() {
            let status = match std::fs::read_to_string(operstate_path)?.trim()
            {
                "up" => Some(Status::Up),
                "down" => Some(Status::Down),
                _ => None,
            };
            Ok(status)
        } else {
            // TODO Alert?
            tracing::error!("operstate_path not found: {:?}", operstate_path);
            Ok(None)
        }
    }
}

struct Symbols<'a> {
    up: &'a str,
    down: &'a str,
}

pub struct State<'a> {
    prefix: &'a str,
    symbols: Symbols<'a>,
    status: Option<Status>,
}

impl<'a> State<'a> {
    pub fn new(prefix: &'a str) -> Self {
        Self {
            prefix,
            symbols: Symbols {
                up: "<>",
                down: "--",
            },
            status: None,
        }
    }
}

impl<'a> crate::State for State<'a> {
    type Msg = Option<Status>;

    fn update(
        &mut self,
        status: Self::Msg,
    ) -> Result<Option<Vec<Box<dyn crate::Alert>>>> {
        self.status = status;
        let alerts = None;
        Ok(alerts)
    }

    fn display<W: std::io::Write>(&self, mut buf: W) -> Result<()> {
        write!(buf, "{}", self.prefix)?;
        match self.status {
            Some(Status::Up) => {
                write!(buf, "{}", self.symbols.up)?;
            }
            Some(Status::Down) | None => {
                write!(buf, "{}", self.symbols.down)?;
            }
        }
        writeln!(buf)?;
        Ok(())
    }
}

pub fn run(interval: Duration, interface: &str, prefix: &str) -> Result<()> {
    let events = crate::clock::new(interval);
    let reader = crate::net::if_operstate::reader(interface);
    let state = crate::net::if_operstate::State::new(prefix);
    let mut stdout = std::io::stdout().lock();
    crate::pipeline(events, reader, state, &mut stdout)
}

pub fn reader<'a, E>(
    interface: &'a str,
) -> Box<dyn 'a + Fn(E) -> Result<Option<Status>>> {
    let path = path(interface);
    tracing::info!("operstate path: {:?}", &path);
    Box::new(move |_| Status::read(&path))
}

fn path(interface: &str) -> PathBuf {
    ["/sys/class/net", interface, "operstate"].iter().collect()
}
