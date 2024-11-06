use clap::Parser;

use stamon::feeds::net;

#[derive(Debug, clap::Subcommand)]
enum IFKind {
    Wifi,
    Eth,
}

#[derive(Debug, clap::Parser)]
struct Cli {
    /// Log level.
    #[clap(short, long, default_value_t = tracing::Level::INFO)]
    log_level: tracing::Level,

    interface: String,

    #[clap(subcommand)]
    interface_kind: IFKind,

    #[clap(long = "interval", short = 'i', default_value = "5")]
    interval: u64,

    #[clap(long = "prefix", default_value = "net ")]
    prefix: String,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    stamon::logger::init(cli.log_level)?;
    tracing::info!("cli: {:#?}", &cli);
    let Cli {
        interface,
        interval,
        prefix,
        interface_kind,
        ..
    } = &cli;
    let interval = std::time::Duration::from_secs(*interval);
    match interface_kind {
        IFKind::Wifi => net::wifi_link_qual::run(interval, interface, prefix),
        IFKind::Eth => net::if_operstate::run(interval, interface, prefix),
    }
}
