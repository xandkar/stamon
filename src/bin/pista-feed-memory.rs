use clap::Parser;

#[derive(Debug, Parser)]
struct Cli {
    #[clap(long = "interval", short = 'i', default_value = "5")]
    interval: u64,

    #[clap(long = "prefix", default_value = "m ")]
    prefix: String,
}

fn main() -> anyhow::Result<()> {
    pista_feeds::log::init()?;
    let cli = Cli::parse();
    tracing::info!("Cli: {:?}", &cli);
    pista_feeds::mem::run(
        &cli.prefix,
        std::time::Duration::from_secs(cli.interval),
    )
}
