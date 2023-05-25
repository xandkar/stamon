use anyhow::{anyhow, Result};
use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[clap(long = "prefix", default_value = "v ")]
    prefix: String,

    #[clap(long = "symbol-mic-on", default_value = "!")]
    symbol_mic_on: String,

    #[clap(long = "symbol-mic-off", default_value = " ")]
    symbol_mic_off: String,
}

fn main() -> Result<()> {
    pista_feeds::log::init()?;
    let cli = Cli::parse();
    let mut stdout = std::io::stdout().lock();

    let mut state = pista_feeds::pulseaudio::State::new()?;
    for update_result in pista_feeds::pulseaudio::Updates::new()?.iter()? {
        match update_result {
            Ok(update) => match state.update(update) {
                Ok(()) => {
                    if let Err(e) = state.write(
                        &mut stdout,
                        &cli.prefix,
                        &cli.symbol_mic_on,
                        &cli.symbol_mic_off,
                    ) {
                        tracing::error!("Failed to write to stdout: {:?}", e);
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to update state: {:?}", e);
                }
            },
            Err(e) => {
                tracing::error!("Failed to read event: {:?}", e);
            }
        }
    }
    Err(anyhow!("Unexpected exit of 'pactl subscribe'"))
}
