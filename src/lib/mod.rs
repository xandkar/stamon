pub mod backlight;
pub mod bluetooth;
pub mod disk;
pub mod math;
pub mod mem;
pub mod mpd;
pub mod net;
pub mod process;
pub mod pulseaudio;
pub mod upower;
pub mod weather;

use std::time::Duration;

use anyhow::{anyhow, Result};

// TODO Everything must implement State
//      - State.new(cfg) --> State
//      - State.update(msg) --> Option<Vec<Alert>>
//      - State.display(buf) --> ()
//      which can then be tested by giving a sequence of updates and examining
//      the data written to the buffer.

pub trait State {
    type Msg;

    fn update(
        &mut self,
        update: Self::Msg,
        // TODO Pass alerts as an iterator?
        // XXX Wrap in Option to avoid allocating a Vec in the common case.
    ) -> Result<Option<Vec<Box<dyn Alert>>>>;

    fn display<W: std::io::Write>(&self, buf: W) -> Result<()>;
}

pub trait Alert {
    fn send(&self) -> Result<()>;
}

pub struct Clock {
    interval: Duration,
}

impl Clock {
    pub fn new(interval: Duration) -> Self {
        Self { interval }
    }
}

impl Iterator for Clock {
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        std::thread::sleep(self.interval);
        Some(())
    }
}

pub fn pipeline<Event, Msg>(
    events: impl Iterator<Item = Event>,
    read: impl Fn(Event) -> Result<Msg>,
    mut state: impl State<Msg = Msg>,
    mut buf: impl std::io::Write,
) -> Result<()> {
    for event in events {
        match read(event) {
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

// TODO Move to tracing module
pub fn tracing_init() -> Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(
                    tracing_subscriber::filter::LevelFilter::INFO.into(),
                )
                .from_env()?,
        )
        .with_writer(std::io::stderr)
        .with_file(true)
        .with_line_number(true)
        .with_timer(tracing_subscriber::fmt::time::LocalTime::rfc_3339())
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}
