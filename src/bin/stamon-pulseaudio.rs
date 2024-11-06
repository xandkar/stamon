use clap::Parser;

#[derive(Parser, Debug)]
struct Cli {
    /// Log level.
    #[clap(short, long, default_value_t = tracing::Level::INFO)]
    log_level: tracing::Level,

    #[clap(long = "prefix", default_value = "v ")]
    prefix: String,

    #[clap(long = "symbol-mic-on", default_value = "!")]
    symbol_mic_on: String,

    #[clap(long = "symbol-mic-off", default_value = " ")]
    symbol_mic_off: String,
}

impl Cli {
    fn symbols(&self) -> stamon::feeds::pulseaudio::Symbols {
        stamon::feeds::pulseaudio::Symbols {
            prefix: &self.prefix,
            mic_on: &self.symbol_mic_on,
            mic_off: &self.symbol_mic_off,
            mute: "  X  ",
            equal: "=",
            approx: "~",
        }
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    stamon::logger::init(cli.log_level)?;
    tracing::info!("cli: {:#?}", &cli);
    stamon::feeds::pulseaudio::run(cli.symbols())
}
