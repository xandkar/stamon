use clap::Parser;

#[derive(Debug, Parser)]
struct Cli {
    #[clap(long = "interval", short = 'i', default_value = "5")]
    interval: u64,

    #[clap(long = "prefix", default_value = "m ")]
    prefix: String,
}

fn main() -> anyhow::Result<()> {
    pista_feeds::logger::init()?;
    let cli = Cli::parse();
    tracing::info!("cli: {:#?}", &cli);
    pista_feeds::feeds::mem::run(
        &cli.prefix,
        std::time::Duration::from_secs(cli.interval),
    )
}
