use clap::Parser;

#[derive(Parser, Debug)]
struct Cli {
    #[clap(long = "device", default_value = "intel_backlight")]
    device: String,

    #[clap(long = "prefix", default_value = "☀ ")]
    prefix: String,
}

fn main() -> anyhow::Result<()> {
    pista_feeds::logger::init()?;
    let cli = Cli::parse();
    tracing::info!("cli: {:?}", &cli);
    pista_feeds::feeds::backlight::run(&cli.device, &cli.prefix)
}
