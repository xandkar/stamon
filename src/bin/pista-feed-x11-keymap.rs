use std::time::Duration;

use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
struct Cli {
    #[clap(long = "interval", short = 'i', default_value = "1.0")]
    interval: f32,

    #[clap(long = "prefix", short = 'p', default_value = "")]
    prefix: String,
}

fn main() -> Result<()> {
    pista_feeds::logger::init()?;
    let cli = Cli::parse();
    tracing::info!("cli: {:?}", &cli);
    pista_feeds::feeds::x11::run(
        &cli.prefix,
        Duration::from_secs_f32(cli.interval),
    )
}
