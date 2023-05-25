use anyhow::Result;
use clap::Parser;

use pista_feeds::x11::X11;

#[derive(Debug, Parser)]
struct Cli {
    #[clap(long = "interval", short = 'i', default_value = "1.0")]
    interval: f32,

    #[clap(long = "prefix", short = 'p', default_value = "")]
    prefix: String,
}

fn main() -> Result<()> {
    pista_feeds::log::init()?;
    let cli = Cli::parse();
    tracing::info!("cli: {:?}", &cli);
    let x11 = X11::init()?;
    let mut stdout = std::io::stdout().lock();
    loop {
        match x11.keymap() {
            Ok(symbol) => {
                if let Err(e) = {
                    use std::io::Write;
                    writeln!(stdout, "{}{}", &cli.prefix, symbol)
                } {
                    tracing::error!("Failed to write to stdout: {:?}", e);
                }
            }
            Err(err) => {
                tracing::error!("Failure to lookup keymap: {:?}", err);
            }
        }
        std::thread::sleep(std::time::Duration::from_secs_f32(cli.interval));
    }
}
