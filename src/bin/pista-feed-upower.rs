use anyhow::Result;
use clap::Parser;

const DEFAULT_ALERTS: [u64; 14] =
    [100, 75, 50, 40, 30, 25, 20, 15, 10, 5, 4, 3, 2, 1];

#[derive(clap::Parser, Debug)]
struct Cli {
    #[clap(long = "prefix", default_value = "⚡ ")]
    prefix: String,

    #[clap(long = "alert", short)]
    alerts: Vec<u64>,
}

impl Cli {
    fn parse_and_validate() -> Self {
        let mut cli = Cli::parse();
        tracing::info!("cli init: {:?}", &cli);
        // TODO: Is there really no way to define a default_value_t for a Vec<T>?
        // "`Vec<u64>` cannot be formatted with the default formatter" when
        // "default_value_t = DEFAULT_ALERTS.to_vec()"
        let alert_triggers = if cli.alerts.is_empty() {
            &DEFAULT_ALERTS[..]
        } else {
            &cli.alerts[..]
        };
        // TODO Integrate this validation with clap derive somehow:
        if let Some(n) = alert_triggers.iter().find(|n| **n > 100) {
            panic!("Alert value out of percentage range: {:?}", n)
        }
        cli.alerts = alert_triggers.to_vec();
        tracing::info!("cli with defaults: {:?}", &cli);
        cli
    }
}

fn main() -> Result<()> {
    pista_feeds::log::init()?;
    let cli = Cli::parse_and_validate();
    pista_feeds::upower::run(&cli.prefix, &cli.alerts[..])
}
