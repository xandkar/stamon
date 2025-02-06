use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
struct Cli {
    /// Log level.
    #[clap(short, long, default_value_t = tracing::Level::INFO)]
    log_level: tracing::Level,

    #[clap(long = "interval", short = 'i', default_value = "5")]
    interval: u64,

    #[clap(long = "prefix", default_value = "d ")]
    prefix: String,

    #[clap(long = "postfix", default_value = "")]
    postfix: String,

    #[clap(default_value = "/")]
    path: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    stamon::logger::init(cli.log_level)?;
    tracing::info!("cli: {:#?}", &cli);
    stamon::feeds::disk::run(
        &cli.prefix,
        &cli.postfix,
        std::time::Duration::from_secs(cli.interval),
        &cli.path,
    )
}
