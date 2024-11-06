#[cfg(test)]
mod tests;

use std::{
    net::{IpAddr, SocketAddr},
    time::Duration,
};

use anyhow::Result;

#[derive(Debug)]
pub struct Symbols<'a> {
    pub prefix: &'a str,
    pub postfix: &'a str,
    pub state_play: &'a str,
    pub state_pause: &'a str,
    pub state_stop: &'a str,
    pub state_off: &'a str,
    pub pct_when_stopped: &'a str,
    pub pct_when_streaming: &'a str,
    pub pct_when_off: &'a str,
}

#[derive(Debug)]
pub struct State<'a> {
    symbols: Symbols<'a>,
    status: Option<mpd::status::Status>,
}

impl<'a> State<'a> {
    fn new(symbols: Symbols<'a>) -> Self {
        Self {
            symbols,
            status: None,
        }
    }

    fn display_time<W: std::io::Write>(&self, mut buf: W) -> Result<()> {
        // XXX Ensure constant width 8:
        match self.status.as_ref().map(|s| (s.state, s.elapsed)) {
            None | Some((mpd::status::State::Stop, _) | (_, None)) => {
                write!(buf, "   --:--")?
            }
            Some((_, Some(e))) => {
                let s = e.as_secs(); // total seconds
                let h = s / 3600; // whole hours
                let s = s - (h * 60 * 60); // seconds (beyond hours)
                let m = s / 60; // minutes (beyond hours)
                let s = s - (m * 60); // seconds (beyond minutes)
                match (h, m, s) {
                    (0, m, s) => write!(buf, "   {:02.0}:{:02.0}", m, s)?,
                    (h, m, s) => {
                        write!(buf, "{:02.0}:{:02.0}:{:02.0}", h, m, s)?
                    }
                }
            }
        };
        Ok(())
    }

    fn display_percentage<W: std::io::Write>(
        &self,
        mut buf: W,
    ) -> Result<()> {
        let sym = &self.symbols;
        // XXX Ensure constant width 4:
        // TODO Tests
        match self
            .status
            .as_ref()
            .map(|s| (s.state, s.duration, s.elapsed))
        {
            None => {
                write!(buf, "{:>4}", sym.pct_when_off)?;
            }
            Some((mpd::status::State::Stop, _, _)) => {
                write!(buf, "{:>4}", sym.pct_when_stopped)?;
            }
            Some((_, None, Some(_))) => {
                write!(buf, "{:>4}", sym.pct_when_streaming)?;
            }
            Some((_, Some(tot), Some(cur))) => {
                let tot = tot.as_secs_f32();
                let cur = cur.as_secs_f32();
                match crate::math::percentage_round(cur, tot) {
                    Some(pct) => write!(buf, "{:3.0}%", pct)?,
                    None => write!(buf, "---%")?,
                }
            }
            Some((s, d, e)) => {
                tracing::warn!(
                    "Unexpected combination in status: state:{:?}, \
                            duration:{:?}, \
                            elapsed:{:?}",
                    s,
                    d,
                    e
                );
                write!(buf, " ???")?; // TODO User-configurable?
            }
        };
        Ok(())
    }
}

impl<'a> crate::pipeline::State for State<'a> {
    type Event = Option<mpd::status::Status>;

    fn update(
        &mut self,
        status_opt: Self::Event,
    ) -> Result<Option<Vec<crate::alert::Alert>>> {
        self.status = status_opt;
        Ok(None)
    }

    fn display<W: std::io::Write>(&mut self, mut buf: W) -> Result<()> {
        let sym = &self.symbols;
        let state: &str = match self.status.as_ref().map(|s| s.state) {
            None => sym.state_off,
            Some(mpd::status::State::Play) => sym.state_play,
            Some(mpd::status::State::Pause) => sym.state_pause,
            Some(mpd::status::State::Stop) => sym.state_stop,
        };
        // Avoiding allocations by sequentially writing directly to buf.
        write!(buf, "{}{} ", sym.prefix, state)?;
        self.display_time(&mut buf)?;
        write!(buf, " ")?;
        self.display_percentage(&mut buf)?;
        writeln!(buf, "{}", sym.postfix)?;
        Ok(())
    }
}

fn reads(
    interval: Duration,
    addr: SocketAddr,
) -> impl Iterator<Item = Option<mpd::status::Status>> {
    use crate::clock;

    let mut conn_opt: Option<mpd::Client> = None;
    clock::new(interval).map(move |clock::Tick| {
        if conn_opt.is_none() {
            conn_opt = mpd::Client::connect(addr).ok();
        }
        // Above (re)connection attempt could've still failed.
        if let Some(ref mut conn) = conn_opt {
            match conn.status() {
                Ok(status) => Some(status),
                Err(err) => {
                    tracing::error!("Failure to get status: {:?}", err);
                    tracing::warn!(
                        "Connection close result: {:?}",
                        conn.close()
                    );
                    conn_opt = None;
                    None
                }
            }
        } else {
            None
        }
    })
}

pub fn run(
    interval: Duration,
    addr: IpAddr,
    port: u16,
    symbols: Symbols<'_>,
) -> Result<()> {
    let addr = SocketAddr::new(addr, port);
    crate::pipeline::run_to_stdout(reads(interval, addr), State::new(symbols))
}
