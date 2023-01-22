// data sources:
// - type
//   - f: /sys/class/net/$IFACE/wireless/ --> present if wireless
// - status:
//   - f: /proc/net/wireless
//        "The cfg80211 wext compat layer assumes a maximum quality of 70"
//        -- https://git.openwrt.org/?p=project/iwinfo.git;a=blob;f=iwinfo_nl80211.c
//   - f: /sys/class/net/$IFACE/operstate --> up | down
//   - f: /proc/net/fib_trie
//   - f: /proc/net/route
//   - c: ip route list
//   - l: libnetlink
//   - l: https://github.com/achanda/netlink
// - traffic:
//   - f: /proc/net/dev
// - SSID:
//   - c: iwconfig
//   - c: iwgetid
// - TCP buffers
//  - f: /proc/net/tcp
//      > The "tx_queue" and "rx_queue" are the
//      > outgoing and incoming data queue
//      > in terms of kernel memory usage.

use std::io::BufRead; // .lines() method
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use clap::Parser;

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

    #[clap(long = "prefix", default_value = "n ")]
    prefix: String,
}

#[derive(Debug)]
enum EthStatus {
    Up,
    Down,
}

fn main() -> Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(
                    tracing_subscriber::filter::LevelFilter::INFO.into(),
                )
                .from_env()?,
        )
        .with_writer(std::io::stderr)
        .with_file(true)
        .with_line_number(true)
        .with_timer(tracing_subscriber::fmt::time::LocalTime::rfc_3339())
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    let cli = Cli::parse();
    tracing::info!("Parameters: {:?}", &cli);
    let operstate_path: PathBuf =
        ["/sys/class/net", &cli.interface, "operstate"]
            .iter()
            .collect();
    tracing::info!("operstate_path: {:?}", &operstate_path);
    loop {
        match &cli.interface_kind {
            IFKind::Wifi => match wifi_link_quality_pct(&cli.interface) {
                Ok(Some(pct)) => println!("{}{:3}%", &cli.prefix, pct),
                Ok(None) => println!("{}---", &cli.prefix),
                Err(e) => tracing::error!(
                    "Failure to parse link quality for {:?}: {:?}",
                    &cli.interface,
                    e
                ),
            },
            IFKind::Eth => match eth_status(operstate_path.as_path()) {
                Ok(Some(EthStatus::Up)) => println!("{}<>", &cli.prefix),
                Ok(Some(EthStatus::Down)) | Ok(None) => {
                    println!("{}--", &cli.prefix);
                }
                Err(e) => tracing::error!(
                    "Failure to read operstate file for {:?}: {:?}",
                    &cli.interface,
                    e
                ),
            },
        }
        std::thread::sleep(std::time::Duration::from_secs(cli.interval));
    }
}

fn eth_status(operstate_path: &Path) -> Result<Option<EthStatus>> {
    if operstate_path.exists() {
        let status = match std::fs::read_to_string(operstate_path)?.trim() {
            "up" => Some(EthStatus::Up),
            "down" => Some(EthStatus::Down),
            _ => None,
        };
        Ok(status)
    } else {
        Ok(None)
    }
}

fn wifi_link_quality_pct(interface: &str) -> Result<Option<u64>> {
    let path = "/proc/net/wireless";
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let mut line_num = 0;
    for line_result in reader.lines() {
        let line = line_result?;
        line_num += 1;
        if line_num > 2 && line.starts_with(interface) {
            let mut fields = line.split_whitespace();
            let cur = fields
                .nth(2)
                .ok_or_else(|| {
                    anyhow!("Missing link quality in line: {line:?}")
                })
                .and_then(|lq| {
                    lq.parse::<f64>().map_err(|_| {
                        anyhow!("Link quality value invalid: {lq:?}")
                    })
                })?;
            // "The cfg80211 wext compat layer assumes a maximum quality of 70"
            // https://git.openwrt.org/?p=project/iwinfo.git;a=blob;f=iwinfo_nl80211.c
            let max = 70.0;
            let pct = (cur / max) * 100.0;
            return Ok(Some(pct as u64));
        }
    }
    Ok(None)
}
