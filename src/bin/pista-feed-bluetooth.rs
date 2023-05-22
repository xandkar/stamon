use anyhow::Result;
use clap::Parser;

use pista_feeds::bluetooth::DeviceState;

#[derive(Debug, clap::Parser)]
struct Cli {
    #[clap(long = "interval", short = 'i', default_value = "2.0")]
    interval: f64,

    #[clap(long = "prefix", default_value = "ᛒ ")]
    prefix: String,
}

fn main() -> Result<()> {
    pista_feeds::tracing_init()?;
    let cli = Cli::parse();
    tracing::info!("cli: {:?}", &cli);
    let interval = std::time::Duration::from_secs_f64(cli.interval);
    let mut stdout = std::io::stdout().lock();
    loop {
        match DeviceState::read() {
            Err(e) => {
                tracing::error!(
                    "Failed to get bluetooth device state: {:?}",
                    e
                );
            }
            Ok(None) => {
                tracing::warn!("Did not find a bluetooth device");
            }
            Ok(Some(state)) => {
                if let Err(e) = { state.write(&mut stdout, &cli.prefix) } {
                    tracing::error!("Failed to write to stdout: {:?}", e);
                }
            }
        }
        std::thread::sleep(interval);
    }
}
