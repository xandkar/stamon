use anyhow::{anyhow, Result};

#[derive(clap::Parser, Debug)]
struct Cli {
    #[clap(long = "prefix", default_value = "⚡ ")]
    prefix: String,
}

fn main() -> Result<()> {
    pista_feeds::tracing_init()?;
    let cli = {
        use clap::Parser;
        Cli::parse()
    };
    tracing::info!("cli: {:?}", &cli);
    let mut stdout = std::io::stdout().lock();
    let mut message_lines = pista_feeds::upower::run()?;
    let mut messages =
        pista_feeds::upower::Messages::from_output_lines(&mut message_lines);
    let state_aggregates =
        pista_feeds::upower::StateAggregates::from_messages(&mut messages);
    for (direction, percentage) in state_aggregates {
        tracing::debug!(
            "Current: direction={:?}, percentage={:?}",
            direction,
            percentage,
        );
        // TODO Notify on negative state changes.
        if let Err(e) = {
            use std::io::Write;
            writeln!(
                stdout,
                "{}{}{:3.0}%",
                &cli.prefix,
                direction.to_char(),
                percentage.floor() // Show the worst case.
            )
        } {
            tracing::error!("Failed to write to stdout: {:?}", e)
        }
    }
    Err(anyhow!("upower exited"))
}
