use std::{thread::sleep, time::Duration};

use anyhow::{anyhow, Result};

pub mod noaa;
pub mod openweathermap;

#[derive(Debug)]
pub struct Observation {
    pub temp_f: f32,
}

pub trait Observatory {
    // TODO Mutable? Might need it to track custom retry state per Observatory.
    fn fetch(&self) -> Result<Observation>;

    fn module_path(&self) -> &str;
}

pub struct Observations {
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
