// TODO "feeds" submodule
pub mod backlight;
pub mod bluetooth;
pub mod disk;
pub mod mem;
pub mod mpd;
pub mod net;
pub mod pulseaudio;
pub mod upower;
pub mod weather;
pub mod x11;

pub mod clock;
pub mod log;
pub mod math;
pub mod process;

use anyhow::{anyhow, Result};

// TODO Everything must implement State
//      - State.new(cfg) --> State
//      - State.update(msg) --> Option<Vec<Alert>>
//      - State.display(buf) --> ()
//      which can then be tested by giving a sequence of updates and examining
//      the data written to the buffer.

pub trait State {
    type Msg;

    // XXX Alerts wrapped in Option to avoid allocating a Vec in the common case.
    fn update(
        &mut self,
        msg: Self::Msg,
    ) -> Result<Option<Vec<Box<dyn Alert>>>>;

    fn display<W: std::io::Write>(&self, buf: W) -> Result<()>;
}

pub trait Alert {
    fn send(&self) -> Result<()>;
}

pub fn pipeline<Event, Msg>(
    events: impl Iterator<Item = Event>,
    event_to_state_msg: impl Fn(Event) -> Result<Msg>,
    mut state: impl State<Msg = Msg>,
    mut buf: impl std::io::Write,
) -> Result<()> {
    // TODO Redesign for backoff, so it is usable for weather
    //      and potentially other remote source polling.
    for event in events {
        match event_to_state_msg(event) {
            Err(err) => {
                tracing::error!("Reader failed to read: {:?}", err);
            }
            Ok(msg) => match state.update(msg) {
                Err(err) => {
                    tracing::error!("State failed to update: {:?}", err);
                }
                Ok(alerts) => {
                    if let Err(e) = state.display(&mut buf) {
                        tracing::error!("State failed to display: {:?}", e);
                    }
                    if let Some(alerts) = alerts {
                        alerts.iter().for_each(|a| {
                            if let Err(e) = a.send() {
                                tracing::error!(
                                    "Alert failed to send: {:?}",
                                    e
                                );
                            }
                        })
                    }
                }
            },
        }
    }
    Err(anyhow!("Unexpected end of events"))
}

pub fn pipeline_to_stdout<Event, Msg>(
    events: impl Iterator<Item = Event>,
    event_to_state_msg: impl Fn(Event) -> Result<Msg>,
    state: impl State<Msg = Msg>,
) -> Result<()> {
    let stdout = std::io::stdout().lock();
    pipeline(events, event_to_state_msg, state, stdout)
}
