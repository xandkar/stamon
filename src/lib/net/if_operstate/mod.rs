use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::Result;

#[derive(Debug)]
enum Status {
    Up,
    Down,
}

impl Status {
    fn read(operstate_path: &Path) -> Result<Option<Self>> {
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

struct State<'a> {
    prefix: &'a str,
    symbols: Symbols<'a>,
    status: Option<Status>,
}

impl<'a> State<'a> {
    fn new(prefix: &'a str) -> Self {
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
    type Event = Option<Status>;

    fn update(
        &mut self,
        status_opt: Self::Event,
    ) -> Result<Option<Vec<Box<dyn crate::Alert>>>> {
        self.status = status_opt;
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

fn reads(
    interval: Duration,
    interface: &str,
) -> impl Iterator<Item = Option<Status>> {
    use crate::clock;

    let path: PathBuf =
        ["/sys/class/net", interface, "operstate"].iter().collect();
    tracing::info!("operstate path: {:?}", &path);

    clock::new(interval).filter_map(move |clock::Tick| {
        match Status::read(&path) {
            Err(err) => {
                tracing::error!("Failed to read operstate: {:?}", err);
                None
            }
            Ok(status_opt) => Some(status_opt),
        }
    })
}

pub fn run(interval: Duration, interface: &str, prefix: &str) -> Result<()> {
    crate::pipeline_to_stdout(reads(interval, interface), State::new(prefix))
}
