use std::{
    io::BufRead, // .lines()
    time::Duration,
};

use anyhow::{anyhow, Result};

struct Info {
    total: u64,
    available: u64,
}

impl Info {
    fn read() -> Result<Self> {
        let path = "/proc/meminfo";
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let mut total_quant = None;
        let mut avail_quant = None;
        let mut total_units = None;
        let mut avail_units = None;
        for line_result in reader.lines() {
            match (total_quant, avail_quant) {
                (Some(_), Some(_)) => break,
                (_, _) => {
                    let line = line_result?;
                    match line.split_whitespace().collect::<Vec<&str>>()[..] {
                        ["MemTotal:", qty, units] => {
                            total_quant = qty.parse().ok();
                            total_units = Some(units.to_string());
                        }
                        ["MemAvailable:", qty, units] => {
                            avail_quant = qty.parse().ok();
                            avail_units = Some(units.to_string());
                        }
                        _ => (),
                    }
                }
            }
        }
        // Since we only report percentage,
        // we don't care what the units are,
        // only that they're equal.
        if total_units == avail_units {
            Ok(Self {
                total: total_quant.unwrap_or(0),
                available: avail_quant.unwrap_or(0),
            })
        } else {
            Err(anyhow!(
                "Different units in MemTotal:{:?} and MemAvailable:{:?}",
                total_units,
                avail_units
            ))
        }
    }

    fn used(&self) -> u64 {
        self.total - self.available
    }

    fn used_pct(&self) -> Option<u64> {
        let cur = self.used() as f32;
        let tot = self.total as f32;
        crate::math::percentage_ceiling(cur, tot)
    }
}

fn usage() -> Result<Option<u64>> {
    Ok(Info::read()?.used_pct())
}

struct State<'a> {
    prefix: &'a str,
    usage: Option<u64>,
}

impl<'a> State<'a> {
    fn new(prefix: &'a str) -> Self {
        Self {
            prefix,
            usage: None,
        }
    }
}

impl<'a> crate::pipeline::State for State<'a> {
    type Event = Option<u64>;

    fn update(
        &mut self,
        usage: Self::Event,
    ) -> Result<Option<Vec<crate::alert::Alert>>> {
        self.usage = usage;
        Ok(None)
    }

    fn display<W: std::io::Write>(&mut self, mut buf: W) -> Result<()> {
        write!(buf, "{}", self.prefix)?;
        match self.usage {
            None => write!(buf, "----")?,
            Some(pct) => write!(buf, "{:3.0}%", pct)?,
        }
        writeln!(buf)?;
        Ok(())
    }
}

fn reads(interval: Duration) -> impl Iterator<Item = Option<u64>> {
    use crate::clock;
    clock::new(interval).filter_map(|clock::Tick| match usage() {
        Err(err) => {
            tracing::error!("Failed to read memory usage: {:?}", err);
            None
        }
        Ok(usage_opt) => Some(usage_opt),
    })
}

pub fn run(prefix: &str, interval: Duration) -> Result<()> {
    crate::pipeline::run_to_stdout(reads(interval), State::new(prefix))
}
