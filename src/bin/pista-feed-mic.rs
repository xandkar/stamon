use std::collections::HashSet;

use anyhow::{anyhow, Result};
use clap::Parser;
use pista_feeds::pulseaudio::Event;

#[derive(Parser, Debug)]
struct Cli {
    // "i" as in "input"
    #[clap(long = "prefix", default_value = "i")]
    prefix: String,

    #[clap(long = "symbol-on", default_value = "!")]
    symbol_on: String,

    #[clap(long = "symbol-off", default_value = " ")]
    symbol_off: String,
}

fn print<W: std::io::Write>(mut buf: W, cli: &Cli, num: usize) {
    let symbol = if num > 0 {
        &cli.symbol_on
    } else {
        &cli.symbol_off
    };
    if let Err(e) = { writeln!(buf, "{}{}", &cli.prefix, symbol) } {
        tracing::error!("Failed to write to stdout: {:?}", e)
    }
}

fn main() -> Result<()> {
    pista_feeds::tracing_init()?;
    let cli = Cli::parse();
    tracing::info!("cli: {:?}", &cli);
    let mut sources = HashSet::new();
    let mut stdout = std::io::stdout().lock();
    print(&mut stdout, &cli, sources.len());
    for event_result in pista_feeds::pulseaudio::source_outputs_list()?
        .chain(pista_feeds::pulseaudio::source_outputs_subscribe()?)
    {
        match event_result {
            Ok(Event::New(id)) => {
                sources.insert(id);
            }
            Ok(Event::Remove(id)) => {
                sources.remove(&id);
            }
            Err(e) => {
                tracing::error!("Failed to read event: {:?}", e)
            }
        }
        print(&mut stdout, &cli, sources.len());
        tracing::debug!("Sources: {:?}", &sources)
    }
    Err(anyhow!("Unexpected exit of 'pactl subscribe'"))
}
