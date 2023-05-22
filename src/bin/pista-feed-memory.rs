use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
struct Cli {
    #[clap(long = "interval", short = 'i', default_value = "5")]
    interval: u64,

    #[clap(long = "prefix", default_value = "m ")]
    prefix: String,
}

fn main() -> Result<()> {
    pista_feeds::tracing_init()?;
    let cli = Cli::parse();
    tracing::info!("Cli: {:?}", &cli);
    let mut stdout = std::io::stdout().lock();
    loop {
        match pista_feeds::mem::usage() {
            Ok(percentage_in_use) => {
                if let Err(e) = {
                    use std::io::Write;
                    writeln!(
                        stdout,
                        "{}{:3.0}%",
                        &cli.prefix,
                        percentage_in_use.ceil() // Show the worst case - ceiling.
                    )
                } {
                    tracing::error!("Failed to write to stdout: {:?}", e);
                }
            }
            Err(e) => {
                tracing::error!("Failed to read /proc/meminfo: {:?}", e);
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(cli.interval));
    }
}
