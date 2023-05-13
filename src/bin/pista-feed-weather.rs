use std::time::Duration;

use anyhow::{anyhow, Result};
use clap::Parser;

use pista_feeds::weather;

#[derive(Debug, Parser)]
struct Cli {
    station_id: String,

    #[clap(long = "interval", short = 'i', default_value_t = 1800)]
    interval: u64,

    #[clap(long = "summary-file", short = 's')]
    summary_file: Option<std::path::PathBuf>,

    #[clap(long = "app-name", default_value = "pista-feed-weather")]
    app_name: String,

    #[clap(long = "app-version", default_value = env!("CARGO_PKG_VERSION"))]
    app_version: String,

    #[clap(
        long = "app-url",
        default_value = "https://github.com/xandkar/pista-feeds-rs"
    )]
    app_url: String,

    #[clap(
        long = "admin-email",
        default_value = "user-has-not-provided-contact-info"
    )]
    admin_email: String,
}

impl Cli {
    pub fn to_noaa_settings(&self) -> weather::noaa::Settings {
        weather::noaa::Settings {
            user_agent: weather::noaa::UserAgent {
                app_name: self.app_name.to_string(),
                app_version: self.app_version.to_string(),
                app_url: self.app_url.to_string(),
                admin_email: self.admin_email.to_string(),
            },
            station_id: self.station_id.to_string(),
            summary_file: self.summary_file.clone(),
        }
    }
}

pub fn main() -> Result<()> {
    // TODO Async
    // TODO Redesign:
    // - Sequence of implementations of a Weather/Observatory trait:
    //   - noaa
    //   - weather.com
    //   - ...
    // - Execution strategy:
    //   A.
    //     - sorted in order of user preference
    //     - the next tried only if previous fails
    //     - if all fail - backoff, otherwise normal interval
    //   B.
    //     - all spawned in parallel and each handles its own retries and intervals
    //     - aggregate state is asynchronously updated and displayed
    //     - each observation will need a TTL, since async execution could
    //       result in some observations getting much older than others.
    //

    pista_feeds::tracing_init()?;
    let cli = Cli::parse();
    tracing::info!("cli: {:?}", &cli);
    let noaa_settings = cli.to_noaa_settings();
    tracing::info!("noaa_settings: {:?}", &noaa_settings);
    let mut stdout = std::io::stdout().lock();
    let observations = weather::Observations::new(
        weather::noaa::Observatory::new(&noaa_settings)?,
        Duration::from_secs(cli.interval),
        Duration::from_secs(15), // TODO Cli?
    );
    for weather::Observation { temp_f } in observations {
        if let Err(e) = {
            use std::io::Write;
            writeln!(stdout, "{:3.0}°F", temp_f)
        } {
            tracing::error!("Failed to write to stdout: {:?}", e)
        }
    }
    Err(anyhow!("Unexpected exit of observations iterator!"))
}
