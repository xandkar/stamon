use std::time::Duration;

use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
struct Cli {
    /// Log level.
    #[clap(short, long, default_value_t = tracing::Level::INFO)]
    log_level: tracing::Level,

    #[clap(short = 'i', long = "interval", default_value = "2.0")]
    interval: f64,

    #[clap(long = "prefix", default_value = "ᛒ ")]
    prefix: String,

    /// Attempt to count current connections, using the bluetoothctl command.
    #[clap(short, long, default_value_t = false)]
    conn_count: bool,

    /// When attempting to count number of connected devices, we call out to
    /// bluetoothctl, which in some cases may be unresponsive or slow. Timeout
    /// mitigates such situations.
    #[clap(short = 't', long, default_value_t = 1.0)]
    conn_count_timeout: f64,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    stamon::logger::init(cli.log_level)?;
    tracing::info!("cli: {:#?}", &cli);
    stamon::feeds::bluetooth::run(
        &cli.prefix,
        Duration::from_secs_f64(cli.interval),
        cli.conn_count,
        Duration::from_secs_f64(cli.conn_count_timeout),
    )
}
