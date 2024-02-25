use std::time::Duration;

use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
struct Cli {
    #[clap(short, long, default_value_t = false)]
    debug: bool,

    #[clap(short = 'i', long = "interval", default_value = "2.0")]
    interval: f64,

    #[clap(long = "prefix", default_value = "ᛒ ")]
    prefix: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    pista_feeds::logger::init(cli.debug)?;
    tracing::info!("cli: {:#?}", &cli);
    pista_feeds::feeds::bluetooth::run(
        &cli.prefix,
        Duration::from_secs_f64(cli.interval),
    )
}
