use clap::Parser;

use pista_feeds::net;

#[derive(Debug, clap::Subcommand)]
enum IFKind {
    Wifi,
    Eth,
}

#[derive(Debug, clap::Parser)]
struct Cli {
    interface: String,

    #[clap(subcommand)]
    interface_kind: IFKind,

    #[clap(long = "interval", short = 'i', default_value = "5")]
    interval: u64,

    #[clap(long = "prefix", default_value = "net ")]
    prefix: String,
}

fn main() -> anyhow::Result<()> {
    pista_feeds::tracing_init()?;
    let cli = Cli::parse();
    tracing::info!("Cli: {:?}", &cli);
    let Cli {
        interface,
        interval,
        prefix,
        interface_kind,
    } = &cli;
    let interval = std::time::Duration::from_secs(*interval);
    match interface_kind {
        IFKind::Wifi => net::wifi_link_qual::run(interval, interface, prefix),
        IFKind::Eth => net::if_operstate::run(interval, interface, prefix),
    }
}
