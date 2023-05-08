use anyhow::{anyhow, Result};
use notify_rust::{Notification, Urgency};

use pista_feeds::upower::Direction;

const DEFAULT_ALERTS: [u64; 14] =
    [100, 75, 50, 40, 30, 25, 20, 15, 10, 5, 4, 3, 2, 1];

#[derive(clap::Parser, Debug)]
struct Cli {
    #[clap(long = "prefix", default_value = "⚡ ")]
    prefix: String,

    // FIXME "`Vec<u64>` cannot be formatted with the default formatter" when
    //       default_value_t = DEFAULT_ALERTS.to_vec())]
    #[clap(long = "alert")]
    alerts: Vec<u64>,
}

fn main() -> Result<()> {
    pista_feeds::tracing_init()?;
    let cli = {
        use clap::Parser;
        Cli::parse()
    };
    tracing::info!("cli: {:?}", &cli);
    let alerts_init: Vec<u64> = if !cli.alerts.is_empty() {
        cli.alerts.clone()
    } else {
        DEFAULT_ALERTS.to_vec()
    };
    tracing::info!("alerts init: {:?}", &alerts_init);
    let mut alerts = alerts_init.clone();
    let mut stdout = std::io::stdout().lock();
    let mut message_lines = pista_feeds::upower::run()?;
    let mut messages =
        pista_feeds::upower::Messages::from_output_lines(&mut message_lines);
    let state_aggregates =
        pista_feeds::upower::StateAggregates::from_messages(&mut messages);
    for (direction, percentage) in state_aggregates {
        tracing::debug!(
            "Current: direction={:?}, percentage={:?}, alerts={:?}",
            direction,
            percentage,
            &alerts
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
        match direction {
            Direction::Decreasing if !percentage.is_nan() => {
                let (mut triggered, remaining): (Vec<u64>, Vec<u64>) = alerts
                    .iter()
                    .partition(|threshold| threshold > &&(percentage as u64));
                triggered.sort();
                triggered.first().map(|threshold| {
                    // TODO User-specifyable urgency levels:
                    //      - per alert?
                    //      - thresholds?
                    let urgency = Urgency::Normal;
                    let _ = Notification::new()
                        .summary(&format!(
                            "Battery power bellow {}%!",
                            threshold
                        ))
                        .body(&format!("{}%", percentage))
                        .urgency(urgency)
                        .show()
                        .map(|_| {
                            tracing::info!(
                                "Alert notification sent for {} < {}",
                                percentage,
                                threshold
                            );
                        })
                        .map_err(|e| {
                            tracing::error!(
                                "Failed to send alert notification for {} < {}: {:?}",
                                percentage, threshold, e
                            )
                        });
                });
                alerts = remaining;
            }
            _ => alerts = alerts_init.clone(),
        }
    }
    Err(anyhow!("upower exited"))
}
