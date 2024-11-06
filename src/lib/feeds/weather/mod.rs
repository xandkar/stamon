// TODO Async
// TODO Redesign to concurrently poll and compare multiple sources.
//     - all spawned in parallel and each handles its own retries and intervals
//     - aggregate state is asynchronously updated and displayed
//         - possible aggregate functions:
//             - min
//             - mean
//             - max
//             - preferred, in order listed in CLI, but that amounts to strategy A
//     - each observation will need a TTL, since async execution could
//       result in some observations getting much older than others.
//     - combined report for all observatories, written to file
pub mod observatories;

use std::{thread::sleep, time::Duration};

use anyhow::{anyhow, Result};

#[derive(Debug)]
pub struct Observation {
    temp_f: f32,
}

pub trait Observatory {
    // TODO Mutable? Might need it to track custom retry state per Observatory.
    fn fetch(&self) -> Result<Observation>;

    fn module_path(&self) -> &str;
}

struct Observations {
    observatories: Vec<Box<dyn Observatory>>,
    first_iteration: bool,
    interval_init: Duration,
    interval_curr: Duration,
    interval_err_init: Duration,
    interval_err_curr: Duration,
}

impl Observations {
    pub fn new(
        observatories: Vec<Box<dyn Observatory>>,
        interval_norm: Duration,
        interval_err_init: Duration,
    ) -> Result<Self> {
        if observatories.is_empty() {
            Err(anyhow!("no observatories provided"))
        } else {
            Ok(Self {
                first_iteration: true,
                interval_init: interval_norm,
                interval_curr: interval_norm,
                interval_err_init,
                interval_err_curr: interval_err_init,
                observatories,
            })
        }
    }
}

impl Iterator for Observations {
    type Item = Observation;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if !self.first_iteration {
                sleep(self.interval_curr);
            }
            self.first_iteration = false;
            for observatory in &self.observatories {
                match observatory.fetch() {
                    Err(e) => {
                        tracing::error!(
                            "Failure to fetch observation from {:?}: {:?}",
                            observatory.module_path(),
                            e
                        );
                    }
                    Ok(o) => {
                        self.interval_curr = self.interval_init;
                        self.interval_err_curr = self.interval_err_init;
                        return Some(o);
                    }
                }
            }
            self.interval_curr = self.interval_err_curr;
            self.interval_err_curr *= 2;
            tracing::warn!(
                "All observatories failed. Next retry in {} seconds.",
                self.interval_curr.as_secs()
            );
        }
    }
}

struct State {
    temp_f: Option<f32>,
}

impl State {
    fn new() -> Self {
        Self { temp_f: None }
    }
}

impl crate::pipeline::State for State {
    type Event = Observation;

    fn update(
        &mut self,
        Observation { temp_f }: Self::Event,
    ) -> Result<Option<Vec<crate::alert::Alert>>> {
        self.temp_f = Some(temp_f);
        Ok(None)
    }

    fn display<W: std::io::Write>(&mut self, mut buf: W) -> Result<()> {
        match self.temp_f {
            None => writeln!(buf, "---°F")?,
            Some(temp_f) => writeln!(buf, "{:3.0}°F", temp_f)?,
        }
        Ok(())
    }
}

pub fn run(
    interval: Duration,
    observatories: Vec<Box<dyn Observatory>>,
) -> Result<()> {
    let observations = Observations::new(
        observatories,
        interval,
        Duration::from_secs(15), // TODO Cli?
    )?;
    crate::pipeline::run_to_stdout(observations, State::new())
}
