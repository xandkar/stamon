use clap::Parser;

#[derive(Parser, Debug)]
struct Cli {
    #[clap(long, short, default_value_t = false)]
    debug: bool,

    #[clap(long = "device", default_value = "intel_backlight")]
    device: String,

    #[clap(long = "prefix", default_value = "☀ ")]
    prefix: String,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    pista_feeds::logger::init(cli.debug)?;
    tracing::info!("cli: {:#?}", &cli);
    pista_feeds::feeds::backlight::run(&cli.device, &cli.prefix)
}
