use anyhow::{anyhow, Result};

use crate::alert::Alert;

/// Each feed should implement State, so that it can be tested by giving a
/// sequence of updates and examining the data written to the buffer.
pub trait State {
    type Event;

    // XXX Alerts wrapped in Option to avoid allocating a Vec in the common case.
    fn update(&mut self, event: Self::Event) -> Result<Option<Vec<Alert>>>;

    fn display<W: std::io::Write>(&mut self, buf: W) -> Result<()>;
}

pub fn run<Event>(
    events: impl Iterator<Item = Event>,
    mut state: impl State<Event = Event>,
    mut buf: impl std::io::Write,
) -> Result<()> {
    // TODO Redesign for backoff, so it is usable for weather
    //      and potentially other remote source polling.
    for event in events {
        match state.update(event) {
            Err(err) => {
                tracing::error!("State update failed: {:?}", err);
            }
            Ok(alerts) => {
                if let Err(e) = state.display(&mut buf) {
                    tracing::error!("State display failed: {:?}", e);
                }
                if let Some(alerts) = alerts {
                    for a in alerts.iter() {
                        if let Err(e) = a.send() {
                            tracing::error!("Alert send failed: {:?}", e);
                        }
                    }
                }
            }
        }
    }
    Err(anyhow!("Unexpected end of events"))
}

pub fn run_to_stdout<Event>(
    events: impl Iterator<Item = Event>,
    state: impl State<Event = Event>,
) -> Result<()> {
    let stdout = std::io::stdout().lock();
    run(events, state, stdout)
}
