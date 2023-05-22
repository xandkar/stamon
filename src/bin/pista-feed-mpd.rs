use std::str::FromStr;

use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
struct Cli {
    #[clap(long = "addr", short = 'h', default_value = "127.0.0.1")]
    addr: String,

    #[clap(long = "port", short = 'p', default_value = "6600")]
    port: u16,

    #[clap(long = "interval", short = 'i', default_value = "1")]
    interval: u64,

    #[clap(long = "prefix", default_value = "")]
    prefix: String,

    #[clap(long = "postfix", default_value = "")]
    postfix: String,

    #[clap(long = "symbol-play", default_value = ">")]
    symbol_play: String,

    #[clap(long = "symbol-pause", default_value = "=")]
    symbol_pause: String,

    #[clap(long = "symbol-stop", default_value = "-")]
    symbol_stop: String,

    #[clap(long = "pct-when-stop", default_value = "---")]
    pct_when_stop: String,

    #[clap(long = "pct-when-stream", default_value = "~~~")]
    pct_when_stream: String,
}

fn status_to_string(s: mpd::status::Status, c: &Cli) -> String {
    let state = match s.state {
        mpd::status::State::Play => &c.symbol_play,
        mpd::status::State::Pause => &c.symbol_pause,
        mpd::status::State::Stop => &c.symbol_stop,
    };
    let percentage = match (s.state, s.duration, s.elapsed) {
        // TODO Remove cloning?
        (mpd::status::State::Stop, _, _) => c.pct_when_stop.clone(),
        (_, None, Some(_)) => c.pct_when_stream.clone(),
        (_, Some(tot), Some(cur)) => {
            let tot = tot.as_secs_f64();
            let cur = cur.as_secs_f64();
            format!("{:3.0}%", cur / tot * 100.0)
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
            "???".to_string()
        }
    };
    let time = match (s.state, s.elapsed) {
        (mpd::status::State::Stop, _) | (_, None) => "--:--".to_string(),
        (_, Some(e)) => {
            let s = e.as_secs(); // total seconds
            let h = s / 3600; // whole hours
            let s = s - (h * 60 * 60); // seconds (beyond hours)
            let m = s / 60; // minutes (beyond hours)
            let s = s - (m * 60); // seconds (beyond minutes)
            match (h, m, s) {
                (0, m, s) => format!("{:02.0}:{:02.0}", m, s),
                (h, m, s) => format!("{:02.0}:{:02.0}:{:02.0}", h, m, s),
            }
        }
    };
    format!("{state} {time:>8} {percentage:^4}")
}

fn main() -> Result<()> {
    pista_feeds::tracing_init()?;
    let cli = Cli::parse();
    tracing::info!("params: {:?}", &cli);
    let ip_addr = std::net::IpAddr::from_str(&cli.addr)?;
    let addr = std::net::SocketAddr::new(ip_addr, cli.port);
    let mut conn_opt = None;
    let mut stdout = std::io::stdout().lock();
    loop {
        if conn_opt.is_none() {
            conn_opt = mpd::Client::connect(addr).ok();
        }
        if let Some(ref mut conn) = conn_opt {
            match conn.status() {
                Ok(s) => {
                    if let Err(e) = {
                        use std::io::Write;
                        writeln!(
                            stdout,
                            "{}{}{}",
                            &cli.prefix,
                            status_to_string(s, &cli),
                            &cli.postfix
                        )
                    } {
                        tracing::error!("Failed to write to stdout: {:?}", e);
                    }
                }
                Err(e) => {
                    tracing::error!("Failure to get status: {:?}", e);
                    tracing::debug!(
                        "Connection close result: {:?}",
                        conn.close()
                    );
                    conn_opt = None;
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(cli.interval));
    }
}
