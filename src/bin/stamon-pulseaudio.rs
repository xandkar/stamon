use clap::Parser;

#[derive(Parser, Debug)]
struct Cli {
    #[clap(long, short, default_value_t = false)]
    debug: bool,

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
    stamon::logger::init(cli.debug)?;
    tracing::info!("cli: {:#?}", &cli);
    stamon::feeds::pulseaudio::run(cli.symbols())
}
