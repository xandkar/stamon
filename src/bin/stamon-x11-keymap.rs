use std::time::Duration;

use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
struct Cli {
    #[clap(long, short, default_value_t = false)]
    debug: bool,

    #[clap(long = "interval", short = 'i', default_value = "1.0")]
    interval: f32,

    #[clap(long = "prefix", short = 'p', default_value = "")]
    prefix: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    stamon::logger::init(cli.debug)?;
    tracing::info!("cli: {:#?}", &cli);
    stamon::feeds::x11::run(
        &cli.prefix,
        Duration::from_secs_f32(cli.interval),
    )
}
