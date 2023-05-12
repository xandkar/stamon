use std::{thread::sleep, time::Duration};

use anyhow::Result;

pub mod noaa;

#[derive(Debug)]
pub struct Observation {
    pub temp_f: f32,
}

pub trait Observatory
where
    Self: Sized,
{
    // TODO Mutable? Might need it to track custom retry state per Observatory.
    fn fetch(&self) -> Result<Observation>;
}

pub struct Observations<O: Observatory> {
    observatory: O,
    first_iteration: bool,

    // TODO Use Duration?
    interval_init: Duration,
    interval_curr: Duration,
    interval_err_init: Duration,
    interval_err_curr: Duration,
}

impl<O: Observatory> Observations<O> {
    pub fn new(
        observatory: O,
        interval_norm: Duration,
        interval_err_init: Duration,
    ) -> Self {
        Self {
            first_iteration: true,
            interval_init: interval_norm,
            interval_curr: interval_norm,
            interval_err_init,
            interval_err_curr: interval_err_init,
            observatory,
        }
    }
}

impl<O: Observatory> Iterator for Observations<O> {
    type Item = Observation;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if !self.first_iteration {
                sleep(self.interval_curr)
            }
            self.first_iteration = false;
            match self.observatory.fetch() {
                Err(e) => {
                    tracing::error!("Failure to fetch observation: {:?}", e);
                    self.interval_curr = self.interval_err_curr;
                    self.interval_err_curr *= 2;
                    tracing::warn!(
                        "Next retry in {} seconds.",
                        self.interval_curr.as_secs()
                    );
                }
                Ok(o) => {
                    self.interval_curr = self.interval_init;
                    self.interval_err_curr = self.interval_err_init;
                    return Some(o);
                }
            }
        }
    }
}
