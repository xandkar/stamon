use std::{net::IpAddr, str::FromStr, time::Duration};

use anyhow::{anyhow, Result};
use clap::Parser;

#[derive(Debug, Parser)]
struct Cli {
    #[clap(long = "addr", default_value = "127.0.0.1")]
    addr: String,

    #[clap(long = "port", default_value = "6600")]
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

impl Cli {
    fn symbols(&self) -> pista_feeds::mpd::Symbols {
        pista_feeds::mpd::Symbols {
            prefix: &self.prefix,
            postfix: &self.postfix,
            play: &self.symbol_play,
            pause: &self.symbol_pause,
            stop: &self.symbol_stop,
            pct_when_stopped: &self.pct_when_stop,
            pct_when_streaming: &self.pct_when_stream,
        }
    }
}

fn main() -> Result<()> {
    pista_feeds::log::init()?;
    let cli = Cli::parse();
    tracing::info!("params: {:?}", &cli);
    let symbols = cli.symbols();
    let states = pista_feeds::mpd::States::new(
        IpAddr::from_str(&cli.addr)?,
        cli.port,
        Duration::from_secs(cli.interval),
    );
    let mut stdout = std::io::stdout().lock();
    for state in states {
        if let Err(e) = { state.display(&mut stdout, &symbols) } {
            tracing::error!("Failed to write to stdout: {:?}", e);
        }
    }
    Err(anyhow!("Unexpected exit"))
}
