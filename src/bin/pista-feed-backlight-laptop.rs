use anyhow::{anyhow, Result};
use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[clap(long = "device", default_value = "intel_backlight")]
    device: String,

    #[clap(long = "prefix", default_value = "☀ ")]
    prefix: String,
}

fn main() -> Result<()> {
    pista_feeds::tracing_init()?;
    let cli = Cli::parse();
    let mut stdout = std::io::stdout().lock();
    for percentage in
        pista_feeds::backlight::Watcher::new(&cli.device)?.iter()
    {
        match percentage {
            Ok(percentage) => {
                if let Err(e) = {
                    use std::io::Write;
                    writeln!(stdout, "{}{:3.0}%", &cli.prefix, percentage)
                } {
                    tracing::error!("Failed to write to stdout: {:?}", e)
                }
            }
            Err(e) => tracing::error!("Failed update: {:?}", e),
        }
    }
    Err(anyhow!("Backlight watcher exited unexpectedly!"))
}
