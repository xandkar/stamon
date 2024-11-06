use clap::Parser;

#[derive(Debug, Parser)]
struct Cli {
    /// Log level.
    #[clap(short, long, default_value_t = tracing::Level::INFO)]
    log_level: tracing::Level,

    #[clap(long = "interval", short = 'i', default_value = "5")]
    interval: u64,

    #[clap(long = "prefix", default_value = "m ")]
    prefix: String,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    stamon::logger::init(cli.log_level)?;
    tracing::info!("cli: {:#?}", &cli);
    stamon::feeds::mem::run(
        &cli.prefix,
        std::time::Duration::from_secs(cli.interval),
    )
}
