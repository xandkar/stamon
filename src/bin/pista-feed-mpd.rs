use std::str::FromStr;

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

    #[clap(long = "symbol-off", default_value = " ")]
    symbol_off: String,

    #[clap(long = "pct-when-stop", default_value = "   ")]
    pct_when_stop: String,

    #[clap(long = "pct-when-off", default_value = "   ")]
    pct_when_off: String,

    #[clap(long = "pct-when-stream", default_value = "~~~")]
    pct_when_stream: String,
}

impl Cli {
    fn symbols(&self) -> pista_feeds::feeds::mpd::Symbols {
        pista_feeds::feeds::mpd::Symbols {
            prefix: &self.prefix,
            postfix: &self.postfix,
            state_play: &self.symbol_play,
            state_pause: &self.symbol_pause,
            state_stop: &self.symbol_stop,
            state_off: &self.symbol_off,
            pct_when_stopped: &self.pct_when_stop,
            pct_when_streaming: &self.pct_when_stream,
            pct_when_off: &self.pct_when_off,
        }
    }
}

fn main() -> anyhow::Result<()> {
    pista_feeds::logger::init()?;
    let cli = Cli::parse();
    tracing::info!("cli: {:?}", &cli);
    pista_feeds::feeds::mpd::run(
        std::time::Duration::from_secs(cli.interval),
        std::net::IpAddr::from_str(&cli.addr)?,
        cli.port,
        cli.symbols(),
    )
}
