use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
struct Cli {
    #[clap(default_value = "/")]
    path: String,

    #[clap(long = "interval", short = 'i', default_value = "5")]
    interval: u64,

    #[clap(long = "prefix", default_value = "d ")]
    prefix: String,
}

fn main() -> Result<()> {
    pista_feeds::logger::init()?;
    let cli = Cli::parse();
    tracing::info!("cli: {:?}", &cli);
    pista_feeds::feeds::disk::run(
        &cli.prefix,
        std::time::Duration::from_secs(cli.interval),
        &cli.path,
    )
}
