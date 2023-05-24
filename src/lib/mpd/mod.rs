#[cfg(test)]
mod tests;

use std::{
    net::{IpAddr, SocketAddr},
    time::Duration,
};

use anyhow::Result;

pub struct Symbols<'a> {
    pub prefix: &'a str,
    pub postfix: &'a str,
    pub play: &'a str,
    pub pause: &'a str,
    pub stop: &'a str,
    pub pct_when_stopped: &'a str,
    pub pct_when_streaming: &'a str,
}

#[derive(Debug)]
pub struct State {
    status: mpd::status::Status,
}

impl State {
    pub fn display<W: std::io::Write>(
        &self,
        mut buf: W,
        sym: &Symbols,
    ) -> Result<()> {
        let state: &str = match self.status.state {
            mpd::status::State::Play => sym.play,
            mpd::status::State::Pause => sym.pause,
            mpd::status::State::Stop => sym.stop,
        };
        // Avoiding allocations by sequentially writing directly to buf.
        write!(buf, "{}", sym.prefix)?;
        write!(buf, "{}", state)?;
        write!(buf, " ")?;
        self.display_time(&mut buf)?;
        write!(buf, " ")?;
        self.display_percentage(&mut buf, sym)?;
        write!(buf, "{}", sym.postfix)?;
        writeln!(buf)?;
        Ok(())
    }

    fn display_time<W: std::io::Write>(&self, mut buf: W) -> Result<()> {
        // XXX Ensure constant width 8:
        match (self.status.state, self.status.elapsed) {
            (mpd::status::State::Stop, _) | (_, None) => {
                write!(buf, "   --:--")?
            }
            (_, Some(e)) => {
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
        sym: &Symbols,
    ) -> Result<()> {
        // XXX Ensure constant width 4:
        // TODO Tests
        let status = &self.status;
        match (status.state, status.duration, status.elapsed) {
            (mpd::status::State::Stop, _, _) => {
                write!(buf, "{:>4}", sym.pct_when_stopped)?;
            }
            (_, None, Some(_)) => {
                write!(buf, "{:>4}", sym.pct_when_streaming)?;
            }
            (_, Some(tot), Some(cur)) => {
                let tot = tot.as_secs_f32();
                let cur = cur.as_secs_f32();
                match crate::math::percentage_round(cur, tot) {
                    Some(pct) => write!(buf, "{:3.0}%", pct)?,
                    None => write!(buf, "---%")?,
                }
            }
            (s, d, e) => {
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

pub struct States {
    addr: SocketAddr,
    conn: Option<mpd::Client>,
    interval: Duration,
}

impl States {
    pub fn new(addr: IpAddr, port: u16, interval: Duration) -> Self {
        Self {
            addr: std::net::SocketAddr::new(addr, port),
            conn: None,
            interval,
        }
    }
}

impl Iterator for States {
    type Item = State;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            std::thread::sleep(self.interval);
            if self.conn.is_none() {
                self.conn = mpd::Client::connect(self.addr).ok();
            }
            if let Some(ref mut conn) = self.conn {
                match conn.status() {
                    Ok(status) => return Some(State { status }),
                    Err(e) => {
                        tracing::error!("Failure to get status: {:?}", e);
                        tracing::debug!(
                            "Connection close result: {:?}",
                            conn.close()
                        );
                        self.conn = None;
                    }
                }
            }
        }
    }
}
